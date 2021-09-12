use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
};
use indexmap::IndexMap;

use std::cell::{Cell, RefCell};

use super::{note::Id, Note};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct NoteList {
        pub list: RefCell<IndexMap<Id, Note>>,
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
                .values()
                .nth(position as usize)
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

        note.connect_metadata_changed(clone!(@weak self as obj => move |note| {
            if let Some((position, _, _)) = obj.get_full(&note.id()) {
                obj.items_changed(position as u32, 1, 1);
            }
        }));

        {
            let mut list = imp.list.borrow_mut();
            list.insert(note.id(), note);
        }

        self.items_changed(self.n_items() - 1, 0, 1);
    }

    pub fn remove(&self, note_id: &Id) {
        let imp = &imp::NoteList::from_instance(self);

        let removed = {
            let mut list = imp.list.borrow_mut();
            list.shift_remove_full(note_id)
        };

        if let Some((position, _, _)) = removed {
            self.items_changed(position as u32, 1, 0);
        }
    }

    pub fn get(&self, note_id: &Id) -> Option<Note> {
        let imp = &imp::NoteList::from_instance(self);
        imp.list.borrow().get(note_id).cloned()
    }

    fn get_full(&self, note_id: &Id) -> Option<(usize, Id, Note)> {
        let imp = imp::NoteList::from_instance(self);
        imp.list
            .borrow()
            .get_full(note_id)
            .map(|(pos, note_id, room)| (pos, note_id.clone(), room.clone()))
    }

    pub fn iter(&self) -> NoteListIter {
        NoteListIter::new(self.clone())
    }
}

pub struct NoteListIter {
    model: NoteList,
    i: Cell<u32>,
}

impl NoteListIter {
    fn new(model: NoteList) -> Self {
        Self {
            model,
            i: Cell::new(0),
        }
    }
}

impl Iterator for NoteListIter {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.i.get();

        let item = self.model.item(index);
        self.i.set(index + 1);
        item.map(|x| x.downcast::<Note>().unwrap())
    }
}
