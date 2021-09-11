use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    prelude::*,
};
use once_cell::sync::Lazy;

use std::cell::{Cell, RefCell};

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

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("note-metadata-changed", &[], <()>::static_type().into())
                        .build(),
                ]
            });
            SIGNALS.as_ref()
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

        note.connect_metadata_changed(clone!(@weak self as obj => move |_| {
            obj.emit_by_name("note-metadata-changed", &[]).unwrap();
        }));

        {
            let mut list = imp.list.borrow_mut();
            list.push(note);
        }

        self.items_changed(self.n_items() - 1, 0, 1);
    }

    pub fn remove(&self, index: usize) {
        let imp = &imp::NoteList::from_instance(self);

        {
            let mut list = imp.list.borrow_mut();
            list.remove(index);
        }

        self.items_changed(index as u32, 1, 0);
    }

    pub fn connect_note_metadata_changed<F: Fn(&Self) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("note-metadata-changed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
        .unwrap()
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
