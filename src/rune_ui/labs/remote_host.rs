//! Labs › Remote transcription's card while on: the SSH destination each
//! finished dictation goes to, saved as typed once typing pauses.

use rune::{El, col, row, text};
use rune_kit::TextInput;

use super::super::Ui;
use super::super::controls::{DESCRIPTION, Paint};
use crate::desktop::i18n::tr;

/// How wide the server field is inside its card.
const HOST_WIDTH: f32 = 190.0;

pub(super) fn body(ui: &Ui, p: Paint) -> El {
    let field = TextInput::new(&ui.lab_store.host)
        .placeholder("me@server")
        .width(Some(HOST_WIDTH))
        .on_edit(|ui: &mut Ui| &mut ui.lab_store.host)
        .build(p.k)
        .key("remote-transcription-host");
    col().key("remote-transcription").gap(4.0).child(
        row()
            .key("remote-transcription-host-row")
            .gap(8.0)
            .center_y()
            .child(
                text(tr("Server"))
                    .key("label")
                    .font(DESCRIPTION)
                    .color(p.muted()),
            )
            .child(field),
    )
}
