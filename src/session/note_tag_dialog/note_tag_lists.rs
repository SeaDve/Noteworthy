use gtk::glib;

use std::{rc::Rc, slice::Iter};

use super::{NoteTagList, Tag};

#[derive(Debug, Clone, glib::SharedBoxed)]
#[shared_boxed_type(name = "NwtyTagLists")]
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
    pub fn iter(&self) -> Iter<NoteTagList> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn first(&self) -> Option<&NoteTagList> {
        self.0.first()
    }

    /// Append tag on all `NoteTagList`
    pub fn append_on_all(&self, tag: &Tag) {
        for tag_list in self.iter() {
            if tag_list.append(tag.clone()).is_err() {
                log::warn!(
                    "Trying to append an existing tag with name `{}`",
                    tag.name()
                );
            }
        }
    }

    /// Remove tag on all `NoteTagList`
    pub fn remove_on_all(&self, tag: &Tag) {
        for tag_list in self.iter() {
            if tag_list.remove(tag).is_err() {
                log::warn!(
                    "Trying to remove a tag with name `{}` that doesn't exist in the list",
                    tag.name()
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_empty() {
        let note_tag_list_1 = NoteTagList::new();
        note_tag_list_1.append(Tag::new("A")).unwrap();
        let note_tag_list_2 = NoteTagList::new();
        note_tag_list_2.append(Tag::new("A")).unwrap();

        let note_tag_lists = NoteTagLists::from(vec![note_tag_list_1, note_tag_list_2]);
        assert!(!note_tag_lists.is_empty());

        let note_tag_lists = NoteTagLists::default();
        assert!(note_tag_lists.is_empty());
    }

    #[test]
    fn first() {
        let note_tag_list_1 = NoteTagList::new();
        note_tag_list_1.append(Tag::new("A")).unwrap();
        let note_tag_list_2 = NoteTagList::new();
        note_tag_list_2.append(Tag::new("A")).unwrap();

        let note_tag_lists = NoteTagLists::from(vec![note_tag_list_1.clone(), note_tag_list_2]);
        assert_eq!(note_tag_lists.first(), Some(&note_tag_list_1));
    }

    #[test]
    fn append_on_all() {
        let note_tag_list_1 = NoteTagList::new();
        note_tag_list_1.append(Tag::new("A")).unwrap();

        let note_tag_list_2 = NoteTagList::new();
        note_tag_list_2.append(Tag::new("A")).unwrap();

        let note_tag_lists =
            NoteTagLists::from(vec![note_tag_list_1.clone(), note_tag_list_2.clone()]);
        let tag = Tag::new("B");
        note_tag_lists.append_on_all(&tag);

        assert!(note_tag_list_1.contains(&tag));
        assert!(note_tag_list_2.contains(&tag));
    }

    #[test]
    fn remove_on_all() {
        let tag = Tag::new("B");

        let note_tag_list_1 = NoteTagList::new();
        note_tag_list_1.append(Tag::new("A")).unwrap();
        note_tag_list_1.append(tag.clone()).unwrap();

        let note_tag_list_2 = NoteTagList::new();
        note_tag_list_2.append(Tag::new("A")).unwrap();
        note_tag_list_2.append(tag.clone()).unwrap();

        let note_tag_lists =
            NoteTagLists::from(vec![note_tag_list_1.clone(), note_tag_list_2.clone()]);
        note_tag_lists.remove_on_all(&tag);

        assert!(!note_tag_list_1.contains(&tag));
        assert!(!note_tag_list_2.contains(&tag));
    }
}
