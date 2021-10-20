use gtk::glib::{self, GBoxed};

use crate::model::NoteTagList;

#[derive(Debug, Clone, GBoxed)]
#[gboxed(type_name = "NwtyTagLists")]
pub struct NoteTagLists(Vec<NoteTagList>);

impl From<Vec<NoteTagList>> for NoteTagLists {
    fn from(vec: Vec<NoteTagList>) -> Self {
        Self(vec)
    }
}

impl Default for NoteTagLists {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl NoteTagLists {
    pub fn iter(&self) -> std::slice::Iter<NoteTagList> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn first(&self) -> Option<&NoteTagList> {
        self.0.first()
    }
}
