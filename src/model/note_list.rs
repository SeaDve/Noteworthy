use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
};
use indexmap::IndexMap;

use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
};

use super::{Note, NoteId};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct NoteList {
        pub list: RefCell<IndexMap<NoteId, Note>>,
        pub unsaved_notes: RefCell<HashSet<Note>>,
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
                .get_index(position as usize)
                .map(|(_, v)| v.upcast_ref::<glib::Object>())
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
        let imp = imp::NoteList::from_instance(self);

        note.connect_metadata_changed(clone!(@weak self as obj => move |note| {
            if let Some(position) = obj.get_index_of(&note.id()) {
                obj.items_changed(position as u32, 1, 1);
            }
        }));

        note.connect_is_saved_notify(clone!(@weak self as obj => move |note| {
            let imp = imp::NoteList::from_instance(&obj);
            let mut unsaved_notes = imp.unsaved_notes.borrow_mut();

            if note.is_saved() {
                let res = unsaved_notes.remove(note);
                log::info!("Removed unsaved note with ret `{}`", res);
            } else {
                let res = unsaved_notes.insert(note.clone());
                log::info!("Inserted unsaved note with ret `{}`", res);
            }
        }));

        {
            let mut list = imp.list.borrow_mut();
            list.insert(note.id(), note);
        }

        self.items_changed(self.n_items() - 1, 0, 1);
    }

    pub fn remove(&self, note_id: &NoteId) {
        let imp = imp::NoteList::from_instance(self);

        let removed = {
            let mut list = imp.list.borrow_mut();
            list.shift_remove_full(note_id)
        };

        if let Some((position, _, _)) = removed {
            self.items_changed(position as u32, 1, 0);
        }
    }

    pub fn get(&self, note_id: &NoteId) -> Option<Note> {
        let imp = imp::NoteList::from_instance(self);
        imp.list.borrow().get(note_id).cloned()
    }

    pub fn get_index_of(&self, note_id: &NoteId) -> Option<usize> {
        let imp = imp::NoteList::from_instance(self);
        imp.list.borrow().get_index_of(note_id)
    }

    pub fn take_unsaved_notes(&self) -> HashSet<Note> {
        let imp = imp::NoteList::from_instance(self);
        imp.unsaved_notes.take()
    }

    pub fn iter(&self) -> Iter {
        Iter::new(self.clone())
    }
}

pub struct Iter {
    model: NoteList,
    i: Cell<u32>,
}

impl Iter {
    const fn new(model: NoteList) -> Self {
        Self {
            model,
            i: Cell::new(0),
        }
    }
}

impl Iterator for Iter {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.i.get();

        let item = self.model.item(index);
        self.i.set(index + 1);
        item.map(|x| x.downcast::<Note>().unwrap())
    }
}
