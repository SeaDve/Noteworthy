use gtk::glib::{self, GSharedBoxed};

use std::rc::Rc;

use super::NoteTagList;

#[derive(Debug, Clone, GSharedBoxed)]
#[gshared_boxed(type_name = "NwtyTagLists")]
pub struct NoteTagLists(Rc<Vec<NoteTagList>>);

impl From<Vec<NoteTagList>> for NoteTagLists {
    fn from(vec: Vec<NoteTagList>) -> Self {
        Self(Rc::new(vec))
    }
}

impl Default for NoteTagLists {
    fn default() -> Self {
        Self(Rc::new(Vec::new()))
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
