use gtk::glib;

use super::Tag;

#[derive(Debug, Clone, glib::Boxed, PartialEq)]
#[boxed_type(name = "NwtySidebarViewSwitcherType")]
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
