use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use std::cell::RefCell;

use super::Note;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct NoteList {
        pub list: RefCell<Vec<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteList {
        const NAME: &'static str = "NwtyNoteList";
        type Type = super::NoteList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for NoteList {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl ListModelImpl for NoteList {
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
    pub struct NoteList(ObjectSubclass<imp::NoteList>)
        @implements gio::ListModel;
}

impl NoteList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NoteList.")
    }

    pub fn append(&self, note: Note) {
        let imp = &imp::NoteList::from_instance(self);

        {
            let mut list = imp.list.borrow_mut();
            list.push(note);
        }

        self.items_changed(self.n_items() + 1, 0, 1);
    }

    pub fn remove(&self, index: usize) {
        let imp = &imp::NoteList::from_instance(self);

        {
            let mut list = imp.list.borrow_mut();
            list.remove(index);
        }

        self.items_changed(index as u32, 1, 0);
    }

    // pub fn find(&self, note: Note) -> Option<usize> {
    //     let imp = imp::NoteList::from_instance(self);
    //     let list = imp.list.borrow();
    //     list.iter().position(|other_note| {
    //         note == other_note
    //     })
    // }

    // pub fn find_with_equal_func(
    //     &self,
    //     note: Note,
    //     equal_func: impl FnMut(&Note) -> bool,
    // ) -> Option<usize> {
    //     let imp = &imp::NoteList::from_instance(self);
    //     let list = imp.list.borrow();
    //     list.iter().position(equal_func)
    // }
}
