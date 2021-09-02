use gtk::{glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;

use std::{fs, path::PathBuf};

use crate::{
    session::note::{LocalNote, Note, NoteList},
    Result,
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LocalNotesManager {
        pub directory: OnceCell<String>,
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
                    glib::ParamSpec::new_string(
                        "directory",
                        "Directory",
                        "Where the notes are stored",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "directory" => {
                    let directory = value.get().unwrap();
                    obj.set_directory(directory);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "directory" => obj.directory().to_value(),
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
    pub fn new(directory: &str) -> Self {
        glib::Object::new::<Self>(&[("directory", &directory.to_string())])
            .expect("Failed to create LocalNotesManager.")
    }

    fn set_directory(&self, directory: &str) {
        let imp = imp::LocalNotesManager::from_instance(self);
        imp.directory.set(directory.to_string()).unwrap();
    }

    pub fn directory(&self) -> String {
        let imp = imp::LocalNotesManager::from_instance(self);
        imp.directory.get().unwrap().clone()
    }

    pub fn note_list(&self) -> NoteList {
        let imp = imp::LocalNotesManager::from_instance(self);
        imp.note_list
            .get_or_init(|| self.retrive_notes().unwrap())
            .clone()
    }

    fn retrive_notes(&self) -> Result<NoteList> {
        let directory = self.directory();
        let paths = fs::read_dir(directory)?;

        let note_list = NoteList::new();

        for path in paths.flatten() {
            let path = path.path();
            let note = LocalNote::new(path.as_path());
            note_list.append(note.upcast());
        }

        Ok(note_list)
    }

    pub fn create_note(&self, title: &str) -> Result<Note> {
        let mut file_path = PathBuf::from(self.directory());
        file_path.push(title);
        file_path.set_extension("md");

        let mut count = 1;
        let new_note = loop {
            if !file_path.exists() {
                break LocalNote::new(&file_path);
            }

            file_path.set_file_name(format!("{} ({})", title, count));
            log::info!("File exists");
            count += 1;
        };

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
        fs::remove_file(note.path())?;

        log::info!("Deleted note {}", note.path());

        Ok(())
    }
}
