use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;

use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    session::note::{LocalNote, Note, NoteList},
    Result,
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LocalNotesManager {
        pub path: RefCell<Option<gio::File>>,
        pub note_list: OnceCell<NoteList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocalNotesManager {
        const NAME: &'static str = "NwtyLocalNotesManager";
        type Type = super::LocalNotesManager;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for LocalNotesManager {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "path",
                        "Path",
                        "Path where the notes are stored",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "note-list",
                        "Note List",
                        "List of notes",
                        NoteList::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "path" => {
                    let path = value.get().unwrap();
                    self.path.replace(path);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "path" => self.path.borrow().to_value(),
                "note-list" => obj.note_list().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct LocalNotesManager(ObjectSubclass<imp::LocalNotesManager>);
}

impl LocalNotesManager {
    pub fn new(path: &Path) -> Self {
        let file = gio::File::for_path(path);

        glib::Object::new::<Self>(&[("path", &file)]).expect("Failed to create LocalNotesManager.")
    }

    pub fn set_path(&self, path: &Path) {
        let file = gio::File::for_path(path);
        self.set_property("path", Some(file)).unwrap();
    }

    pub fn path(&self) -> PathBuf {
        self.property("path")
            .unwrap()
            .get::<Option<gio::File>>()
            .unwrap()
            .unwrap()
            .path()
            .unwrap()
    }

    pub fn note_list(&self) -> NoteList {
        let imp = imp::LocalNotesManager::from_instance(self);
        imp.note_list
            .get_or_init(|| self.retrive_notes().unwrap())
            .clone()
    }

    fn retrive_notes(&self) -> Result<NoteList> {
        let paths = fs::read_dir(self.path())?;
        let note_list = NoteList::new();

        for path in paths.flatten() {
            let path = path.path();
            let file = gio::File::for_path(path);
            let note = LocalNote::load_from_file(&file)?;
            note_list.append(note.upcast());
        }

        Ok(note_list)
    }

    pub fn create_note(&self, title: &str) -> Result<Note> {
        let mut file_path = self.path();
        let file_name = format!("{} {}", title, chrono::Local::now().format("%H:%M:%S"));
        file_path.push(file_name);
        file_path.set_extension("md");

        let file = gio::File::for_path(file_path.display().to_string());
        let new_note = LocalNote::from_file(&file)?;

        log::info!("Created note {}", new_note.file().path().unwrap().display());

        let new_note_upcast: Note = new_note.upcast();
        self.note_list().append(new_note_upcast.clone());

        Ok(new_note_upcast)
    }

    pub fn delete_note(&self, index: usize) -> Result<()> {
        let note_list = self.note_list();
        note_list.remove(index);

        let note = note_list
            .item(index as u32)
            .unwrap()
            .downcast::<LocalNote>()
            .unwrap();

        note.delete().unwrap();

        log::info!("Deleted note {}", note.file().path().unwrap().display());

        Ok(())
    }
}
