//! Labs › Remote transcription's strings, kept beside the main tables so
//! those stay under the source cap. Not yet reviewed by native speakers, like
//! their neighbouring tables.

/// Columns: Polish, Chinese (Simplified), Japanese, German, Spanish,
/// Ukrainian.
#[rustfmt::skip]
pub(super) const TRANSLATIONS: &[(&str, [&str; 6])] = &[
    ("Remote transcription", ["Zdalna transkrypcja", "远程转写", "リモート文字起こし", "Remote-Transkription", "Transcripción remota", "Віддалена транскрипція"]),
    (
        "Transcribe on another computer's Hex over SSH. If it can't answer, Hex transcribes here.",
        [
            "Transkrybuj w Hex na innym komputerze przez SSH. Gdy nie odpowiada, Hex transkrybuje tutaj.",
            "通过 SSH 在另一台电脑的 Hex 上转写。它无法响应时，Hex 在本机转写。",
            "SSH で別のコンピュータの Hex で文字起こしします。応答がないときは Hex がここで文字起こしします。",
            "Transkribiere per SSH mit Hex auf einem anderen Computer. Antwortet er nicht, transkribiert Hex hier.",
            "Transcribe con Hex en otro ordenador por SSH. Si no responde, Hex transcribe aquí.",
            "Транскрибуйте в Hex на іншому комп’ютері через SSH. Якщо він не відповідає, Hex транскрибує тут.",
        ],
    ),
    ("Server", ["Serwer", "服务器", "サーバー", "Server", "Servidor", "Сервер"]),
];

#[rustfmt::skip]
pub(super) const FRENCH: &[(&str, &str)] = &[
    ("Remote transcription", "Transcription à distance"),
    (
        "Transcribe on another computer's Hex over SSH. If it can't answer, Hex transcribes here.",
        "Transcrivez avec Hex sur un autre ordinateur via SSH. S'il ne répond pas, Hex transcrit ici.",
    ),
    ("Server", "Serveur"),
];

#[rustfmt::skip]
pub(super) const PORTUGUESE: &[(&str, &str)] = &[
    ("Remote transcription", "Transcrição remota"),
    (
        "Transcribe on another computer's Hex over SSH. If it can't answer, Hex transcribes here.",
        "Transcreva com o Hex em outro computador por SSH. Se ele não responder, o Hex transcreve aqui.",
    ),
    ("Server", "Servidor"),
];

#[rustfmt::skip]
pub(super) const RUSSIAN: &[(&str, &str)] = &[
    ("Remote transcription", "Удалённая транскрипция"),
    (
        "Transcribe on another computer's Hex over SSH. If it can't answer, Hex transcribes here.",
        "Транскрибируйте в Hex на другом компьютере по SSH. Если он не отвечает, Hex транскрибирует здесь.",
    ),
    ("Server", "Сервер"),
];

/// Hungarian, for the strings above.
#[rustfmt::skip]
pub(super) const HUNGARIAN: &[(&str, &str)] = &[
    ("Remote transcription", "Távoli átírás"),
    (
        "Transcribe on another computer's Hex over SSH. If it can't answer, Hex transcribes here.",
        "Átírás egy másik számítógép Hex-ével SSH-n keresztül. Ha az nem válaszol, a Hex itt ír át.",
    ),
    ("Server", "Szerver"),
];
