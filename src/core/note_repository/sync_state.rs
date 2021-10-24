use gtk::glib::{self, GEnum};

#[derive(Clone, Copy, Debug, PartialEq, GEnum)]
#[genum(type_name = "NwtyNoteRepositorySyncState")]
pub enum SyncState {
    Syncing,
    Pulling,
    Pushing,
    Idle,
}

impl Default for SyncState {
    fn default() -> Self {
        Self::Idle
    }
}
