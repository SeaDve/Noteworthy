mod local_manager;

pub use local_manager::LocalManager;

use super::note::{Note, NotesList};
use crate::error::Error;

pub trait ManagerExt {
    fn retrive_notes(&self) -> Result<NotesList, Error>;
    fn create_note(&self, note: Note);
}
