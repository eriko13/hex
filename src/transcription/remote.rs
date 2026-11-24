//! Labs › Remote transcription: a finished dictation's clip goes to Hex on
//! another computer over SSH (`hex transcribe`), so this one keeps its model
//! unloaded. Any failure is returned for the engine to fall back locally.

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use transcribe_cpp::CancelToken;

use crate::labs::RemoteTranscription;

/// SSH's own connect limit; an unreachable host falls back quickly.
const CONNECT_TIMEOUT_SECONDS: u32 = 4;
/// The remote run's allowance before the clip's own share: a cold remote
/// model may load inside it.
const BASE_DEADLINE: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(10);
/// What `hex transcribe` says when the clip held no speech.
const NO_SPEECH: &str = "no speech was heard";

/// The remote transcript of a 16 kHz mono `clip`, while the lab is on with a
/// host; `None` leaves the clip to the local model.
pub(crate) fn transcribe(clip: &[f32], cancel: &CancelToken) -> Option<Result<String, String>> {
    let target = crate::labs::remote_transcription()?;
    // The remote model hears words in silence ("Thank you."); the local
    // detector skips the round trip, as the local path skips such a clip.
    if crate::transcription::speech_detector::contains_speech(clip) == Some(false) {
        return Some(Ok(String::new()));
    }
    let started = Instant::now();
    let result = run(&target, clip, cancel);
    match &result {
        Ok(_) => tracing::info!(
            host = %target.host,
            audio_ms = clip.len() / 16,
            remote_ms = started.elapsed().as_millis(),
            "transcribed remotely"
        ),
        Err(error) => tracing::warn!(host = %target.host, %error, "remote transcription failed"),
    }
    Some(result)
}

fn run(target: &RemoteTranscription, clip: &[f32], cancel: &CancelToken) -> Result<String, String> {
    let wav = wav_bytes(clip)?;
    let deadline = BASE_DEADLINE + Duration::from_secs_f64(clip.len() as f64 / 16_000.0);
    let mut child = ssh(&target.host)
        .arg(remote_command(&target.hex))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start ssh: {error}"))?;
    let mut stdin = child.stdin.take().ok_or("ssh has no stdin")?;
    let writer = thread::spawn(move || stdin.write_all(&wav));
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let status = wait(&mut child, cancel, deadline);
    let _ = writer.join();
    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();
    let status = status?;
    if status.success() {
        return Ok(stdout.trim().to_owned());
    }
    if stderr.contains(NO_SPEECH) || stdout.contains(NO_SPEECH) {
        return Ok(String::new());
    }
    let message = stderr.trim();
    Err(match status.code() {
        Some(255) => format!("ssh could not reach the host: {message}"),
        Some(code) => format!("remote hex exited with {code}: {message}"),
        None => format!("remote hex was killed: {message}"),
    })
}

/// SSH that never prompts, sharing one connection between dictations so
/// each pays only the remote run, not a new handshake.
fn ssh(host: &str) -> Command {
    let mut command = Command::new("ssh");
    command.args([
        "-o",
        "BatchMode=yes",
        "-o",
        &format!("ConnectTimeout={CONNECT_TIMEOUT_SECONDS}"),
        "-o",
        "ServerAliveInterval=5",
        "-o",
        "ServerAliveCountMax=2",
        "-o",
        "ControlMaster=auto",
        // A short path: the socket's must fit a sockaddr_un.
        "-o",
        "ControlPath=/tmp/hex-ssh-%C",
        "-o",
        "ControlPersist=600",
        "--",
        host,
    ]);
    command
}

/// The remote shell runs `hex transcribe` on the WAV read from stdin, in a
/// directory it removes however the run ends.
fn remote_command(hex: &str) -> String {
    format!(
        "d=$(mktemp -d) || exit 1; cat > \"$d/clip.wav\" && {} transcribe \"$d/clip.wav\"; rc=$?; rm -rf \"$d\"; exit $rc",
        shell_word(hex)
    )
}

/// `path` quoted for a POSIX shell, keeping a leading `~/` so it expands.
fn shell_word(path: &str) -> String {
    match path.strip_prefix("~/") {
        Some(rest) => format!("~/{}", quote(rest)),
        None => quote(path),
    }
}

fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_string(&mut text);
        }
        text
    })
}

fn wait(
    child: &mut Child,
    cancel: &CancelToken,
    deadline: Duration,
) -> Result<std::process::ExitStatus, String> {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(error) => return Err(format!("could not wait for ssh: {error}")),
        }
        let reason = if cancel.is_cancelled() {
            "transcription was abandoned"
        } else if started.elapsed() >= deadline {
            "the remote computer did not answer in time"
        } else {
            thread::sleep(POLL);
            continue;
        };
        let _ = child.kill();
        let _ = child.wait();
        return Err(reason.into());
    }
}

/// `clip` as 16-bit PCM: half the bytes of float samples, and the format
/// every `hex transcribe` reads.
fn wav_bytes(clip: &[f32]) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut bytes = std::io::Cursor::new(Vec::with_capacity(44 + clip.len() * 2));
    let mut writer = hound::WavWriter::new(&mut bytes, spec).map_err(|error| error.to_string())?;
    for sample in clip {
        let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        writer
            .write_sample(pcm)
            .map_err(|error| error.to_string())?;
    }
    writer.finalize().map_err(|error| error.to_string())?;
    Ok(bytes.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_remote_hex_keeps_its_home_and_quotes_the_rest() {
        assert_eq!(shell_word("~/.local/bin/hex"), "~/'.local/bin/hex'");
        assert_eq!(shell_word("/opt/my hex/hex"), "'/opt/my hex/hex'");
        assert_eq!(shell_word("it's"), "'it'\\''s'");
    }

    #[test]
    fn the_remote_command_cleans_up_its_clip() {
        let command = remote_command("hex");
        assert!(command.contains("'hex' transcribe \"$d/clip.wav\""));
        assert!(command.ends_with("rm -rf \"$d\"; exit $rc"));
    }

    #[test]
    fn a_clip_becomes_a_16_bit_wav_hex_can_read() {
        let wav = wav_bytes(&[0.0, 0.5, -1.0, 2.0]).unwrap();
        let reader = hound::WavReader::new(std::io::Cursor::new(wav)).unwrap();
        assert_eq!(reader.spec().sample_rate, 16_000);
        let samples: Vec<i16> = reader.into_samples().map(Result::unwrap).collect();
        assert_eq!(samples, [0, 16_383, -32_767, 32_767]);
    }

    #[test]
    fn an_abandoned_run_kills_ssh() {
        let mut child = Command::new("sleep").arg("5").spawn().unwrap();
        let cancel = CancelToken::new();
        cancel.cancel();
        let started = Instant::now();
        let error = wait(&mut child, &cancel, Duration::from_secs(5)).unwrap_err();
        assert_eq!(error, "transcription was abandoned");
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}
