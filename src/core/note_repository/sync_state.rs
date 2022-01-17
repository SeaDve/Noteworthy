use gtk::glib;

#[derive(Debug, Clone, Copy, PartialEq, glib::Enum)]
#[enum_type(name = "NwtyNoteRepositorySyncState")]
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
