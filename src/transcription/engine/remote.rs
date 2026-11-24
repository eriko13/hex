//! Labs › Remote transcription inside the engine: a finished dictation goes
//! to another computer first, and the local model loads only when that
//! computer can't answer. No live stream opens while it's on, so a parked
//! model stays unloaded.

use std::time::Instant;

use transcribe_cpp::CancelToken;

use super::{Job, Offline, Source, Transcribed, Worker};

/// A finished clip's transcript from another computer, or `None` to
/// transcribe it here. An error falls back here.
pub(super) type Remote = fn(&[f32], &CancelToken) -> Option<Result<String, String>>;

// Tests never reach the SSH host this machine's own settings name; they pass
// their remote to `start_parked_with_remote`.
#[cfg(unix)]
pub(super) const REMOTE: Remote = |clip, cancel| {
    if cfg!(test) {
        return None;
    }
    crate::transcription::remote::transcribe(clip, cancel)
};
#[cfg(not(unix))]
pub(super) const REMOTE: Remote = |_, _| None;

/// Why no live stream opens while finished dictations go elsewhere.
pub(super) const REFUSAL: &str =
    "Labs › Remote transcription sends each finished dictation to another computer";

/// Whether finished dictations go to another computer, asked as each stream
/// would begin.
pub(super) fn on() -> bool {
    !cfg!(test) && crate::labs::remote_transcription().is_some()
}

#[cfg(test)]
impl super::TranscriptionEngine {
    /// Like [`Self::start_parked`], with `remote` answering finished clips.
    fn start_parked_with_remote(
        selection: crate::transcription::models::TranscriptionSelection,
        remote: Remote,
    ) -> color_eyre::eyre::Result<Self> {
        Self::with_slot(super::ModelSlot::parked(selection), remote)
    }
}

impl Worker {
    /// Finishes a dictation on another computer while Labs › Remote
    /// transcription is on, without loading the model here; offline here
    /// otherwise, or when the other computer can't answer.
    pub(super) fn finish(&mut self, clip: Vec<f32>, offline: Offline, source: Source, job: Job) {
        if job.cancel.is_cancelled() {
            job.finish(Err("transcription was abandoned".into()));
            return;
        }
        let started = Instant::now();
        self.start(&job);
        self.running.enter(&job.cancel);
        let source = match (self.remote)(&clip, &job.cancel) {
            None => source,
            Some(Ok(text)) => {
                // A fallback loaded the model; the remote answers again, so
                // its memory is freed until the next fallback.
                self.park();
                job.finish(Ok(Transcribed {
                    text,
                    source: Source::Remote,
                    finish_ms: started.elapsed().as_millis() as u64,
                }));
                return;
            }
            Some(Err(_)) if job.cancel.is_cancelled() => {
                job.finish(Err("transcription was abandoned".into()));
                return;
            }
            Some(Err(reason)) => {
                tracing::warn!(%reason, "remote transcription unusable; transcribing here");
                Source::Fallback(format!("remote transcription failed: {reason}"))
            }
        };
        self.offline(clip, offline, source, started, job);
    }
}

#[cfg(test)]
mod tests {
    use super::super::TranscriptionEngine;
    use super::*;

    fn unusable_selection() -> crate::transcription::models::TranscriptionSelection {
        crate::transcription::models::TranscriptionSelection {
            language: "zz".into(),
            ..Default::default()
        }
    }

    #[test]
    fn a_remote_answer_finishes_the_job_without_loading_the_model() {
        let selection = unusable_selection();
        let engine = TranscriptionEngine::start_parked_with_remote(selection.clone(), |clip, _| {
            Some(Ok(format!("remote {}", clip.len())))
        })
        .unwrap();
        let transcribed = engine
            .finish_with(
                None,
                vec![0.0; 3],
                Box::new(|_, _| panic!("a remote answer must not load the model")),
            )
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(transcribed.text, "remote 3");
        assert_eq!(transcribed.source, Source::Remote);
        assert!(engine.holds(&selection), "the model stays parked");
    }

    #[test]
    fn a_failed_remote_falls_back_to_the_local_model() {
        let engine = TranscriptionEngine::start_parked_with_remote(unusable_selection(), |_, _| {
            Some(Err("down".into()))
        })
        .unwrap();
        let transcribed = engine
            .finish_with(
                None,
                Vec::new(),
                Box::new(|_, _| panic!("a parked engine must reload its model first")),
            )
            .unwrap()
            .wait();
        assert!(
            transcribed
                .unwrap_err()
                .starts_with("the speech model could not be reloaded"),
            "the fallback reloads the local model"
        );
    }

    #[test]
    fn remote_transcription_opens_no_stream() {
        let (mut engine, _commands) = TranscriptionEngine::for_test();
        assert!(engine.streams());
        engine.remote_on = || true;
        assert!(!engine.streams());
        assert!(engine.begin().is_none());
        assert!(engine.begin_yielding().is_none());
        assert_eq!(engine.stream_refusal(), Some(REFUSAL));
    }
}
