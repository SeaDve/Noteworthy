mod local_notes_manager;

pub use local_notes_manager::LocalNotesManager;

use super::note::{Note, NotesList};
use crate::error::Error;

pub trait NotesManagerExt {
    fn retrive_notes(&self) -> Result<NotesList, Error>;
    fn create_note(&self, note: Note);
    fn delete_note(&self, note: Note);
}
