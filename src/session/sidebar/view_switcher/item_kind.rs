use gtk::glib::{self, GBoxed};

use super::Tag;

#[derive(Debug, Clone, GBoxed, PartialEq)]
#[gboxed(type_name = "NwtySidebarViewSwitcherType")]
pub enum ItemKind {
    Separator,
    Category,
    AllNotes,
    EditTags,
    Tag(Tag),
    Trash,
}

impl Default for ItemKind {
    fn default() -> Self {
        Self::AllNotes
    }
}
