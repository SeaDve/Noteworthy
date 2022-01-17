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

use super::{Note, NoteId, Tag};
use crate::core::FileType;

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
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for NoteList {}

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

    /// Try load notes on `directory` with file type of markdown
    pub async fn load_from_dir(directory: &gio::File) -> anyhow::Result<Self> {
        let file_infos = directory
            .enumerate_children_future(
                &gio::FILE_ATTRIBUTE_STANDARD_NAME,
                gio::FileQueryInfoFlags::NONE,
                glib::PRIORITY_HIGH_IDLE,
            )
            .await?;

        let note_list = Self::new();

        for file_info in file_infos.flatten() {
            let file_path = {
                let mut file_path = directory.path().unwrap();
                file_path.push(file_info.name());
                file_path
            };

            log::info!("Loading file `{}`", file_path.display());

            let file = gio::File::for_path(&file_path);

            if FileType::for_file(&file) != FileType::Markdown {
                log::info!(
                    "The file `{}` doesn't have an md extension, skipping...",
                    file_path.display()
                );
                continue;
            }

            // TODO consider using GtkSourceFile here
            // So we could use GtkSourceFileLoader and GtkSourceFileSaver to handle
            // saving and loading, and perhaps reduce allocations on serializing into buffer and
            // deserializiations.
            let note = Note::deserialize(&file).await?;
            note_list.append(note);
        }

        Ok(note_list)
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

        imp.list.borrow_mut().insert(note.id(), note);

        self.items_changed(self.n_items() - 1, 0, 1);
    }

    pub fn remove(&self, note_id: &NoteId) {
        let imp = imp::NoteList::from_instance(self);

        let removed = imp.list.borrow_mut().shift_remove_full(note_id);

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

    /// Clear and get all unsaved notes
    pub fn take_unsaved_notes(&self) -> HashSet<Note> {
        let imp = imp::NoteList::from_instance(self);
        imp.unsaved_notes.take()
    }

    /// Remove tag on `TagList` of all `Note`s
    pub fn remove_tag_on_all(&self, tag: &Tag) {
        for note in self.iter() {
            let note_tag_list = note.metadata().tag_list();

            if let Err(err) = note_tag_list.remove(tag) {
                log::warn!(
                    "Failed to remove tag with name `{}` on `{}`: {:?}",
                    tag.name(),
                    note,
                    err
                );
            }
        }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn remove_tag_on_all() {
        // Gtk has to be initialized when Note::create_default is called since
        // GtkSourceView requires it.
        gtk::init().unwrap();

        let note_list = NoteList::new();
        let tag = Tag::new("A");

        let note_1 = Note::create_default("/home/user");
        let note_1_tag_list = note_1.metadata().tag_list();
        note_1_tag_list.append(tag.clone()).unwrap();
        assert!(note_1_tag_list.contains(&tag));
        note_list.append(note_1);

        let note_2 = Note::create_default("/home/user");
        let note_2_tag_list = note_2.metadata().tag_list();
        note_2_tag_list.append(tag.clone()).unwrap();
        assert!(note_2_tag_list.contains(&tag));
        note_list.append(note_2);

        note_list.remove_tag_on_all(&tag);
        assert!(!note_1_tag_list.contains(&tag));
        assert!(!note_2_tag_list.contains(&tag));
    }
}
