//! Labs › Remote transcription's card while on: the SSH destination each
//! finished dictation goes to, saved trimmed once typing pauses.

use rune::{El, col, row, text};
use rune_kit::TextInput;

use super::super::Ui;
use super::super::controls::{DESCRIPTION, Paint};
use super::Store;
use crate::app_settings::AppSettings;
use crate::desktop::i18n::tr;

/// How wide the server field is inside its card.
const HOST_WIDTH: f32 = 190.0;

impl Store {
    /// Starts the server field at `saved` when Labs opens or its lab turns on.
    pub(super) fn sync_host(&mut self, saved: &str) {
        self.host = rune::TextEdit::new(saved.to_owned());
        self.host_synced = true;
    }

    /// Adopts a server another writer saved, unless the field holds an
    /// unsaved draft.
    pub(super) fn rebase_host(&mut self, before: &str, after: &str) {
        if before != after && (!self.host_synced || self.host.text.trim() == before) {
            self.sync_host(after);
        }
    }

    /// The server `settings` should hold. A lab that's off, or a field this
    /// window never opened, leaves the saved server alone.
    pub(super) fn project_host(&self, settings: &mut AppSettings) {
        if settings.labs.remote_transcription && self.host_synced {
            settings.labs.remote_transcription_host = self.host.text.trim().to_owned();
        }
    }

    /// Whether the server field holds typing the saved settings don't have.
    pub(super) fn host_dirty(&self, settings: &AppSettings) -> bool {
        settings.labs.remote_transcription
            && self.host_synced
            && self.host.text.trim() != settings.labs.remote_transcription_host
    }
}

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
