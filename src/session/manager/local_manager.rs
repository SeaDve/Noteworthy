use gtk::{glib, prelude::*, subclass::prelude::*};

use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
};

use super::Manager;
use crate::{
    error::Error,
    session::note::{LocalNote, Note, NotesList},
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LocalManager {
        directory: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocalManager {
        const NAME: &'static str = "NwtyLocalManager";
        type Type = super::LocalManager;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for LocalManager {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_string(
                    "directory",
                    "Directory",
                    "Where the notes are stored",
                    None,
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "directory" => {
                    let directory = value.get().unwrap();
                    self.directory.replace(Some(directory));
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "directory" => self.directory.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct LocalManager(ObjectSubclass<imp::LocalManager>);
}

impl LocalManager {
    pub fn new(directory: &Path) -> Self {
        glib::Object::new::<Self>(&[("directory", &directory.display().to_string())])
            .expect("Failed to create LocalManager.")
    }

    pub fn directory(&self) -> PathBuf {
        let directory: String = self.property("directory").unwrap().get().unwrap();
        PathBuf::from(directory)
    }
}

impl Manager for LocalManager {
    fn retrive_notes(&self) -> Result<NotesList, Error> {
        let directory = self.directory();
        let paths = fs::read_dir(directory)?;

        let notes_list = NotesList::new();

        for path in paths.flatten() {
            let path = path.path();
            let note = LocalNote::new(path.as_path());
            notes_list.append(note.upcast());
        }

        Ok(notes_list)
    }

    fn create_note(&self, note: Note) {
        unimplemented!()
    }
}
