use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use std::cell::RefCell;

use super::Note;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct NotesList {
        pub list: RefCell<Vec<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotesList {
        const NAME: &'static str = "NwtyNotesList";
        type Type = super::NotesList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for NotesList {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl ListModelImpl for NotesList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Note::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get(position as usize)
                .map(glib::object::Cast::upcast_ref::<glib::Object>)
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct NotesList(ObjectSubclass<imp::NotesList>)
        @implements gio::ListModel;
}

impl NotesList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NotesList.")
    }

    pub fn append(&self, note: Note) {
        let imp = &imp::NotesList::from_instance(self);
        imp.list.borrow_mut().push(note);
    }
}
