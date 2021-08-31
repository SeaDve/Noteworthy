mod local_provider;

pub use local_provider::LocalProvider;

use super::note::{Note, NotesList};
use crate::error::Error;

pub trait Provider {
    fn retrive_notes(&self) -> Result<NotesList, Error>;
}
