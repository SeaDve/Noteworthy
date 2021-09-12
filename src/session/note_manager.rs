use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{Note, NoteList};
use crate::Result;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct NoteManager {
        pub path: OnceCell<gio::File>,
        pub note_list: OnceCell<NoteList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteManager {
        const NAME: &'static str = "NwtyNoteManager";
        type Type = super::NoteManager;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for NoteManager {
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
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "note-list",
                        "Note List",
                        "List of notes",
                        NoteList::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                ]
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
                "path" => {
                    let path = value.get().unwrap();
                    self.path.set(path).unwrap();
                }
                "note-list" => {
                    let note_list = value.get().unwrap();
                    self.note_list.set(note_list).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "path" => self.path.get().to_value(),
                "note-list" => obj.note_list().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct NoteManager(ObjectSubclass<imp::NoteManager>);
}

impl NoteManager {
    pub fn new(path: &Path) -> Self {
        let file = gio::File::for_path(path);

        glib::Object::new::<Self>(&[("path", &file)]).expect("Failed to create NoteManager.")
    }

    pub fn path(&self) -> PathBuf {
        self.property("path")
            .unwrap()
            .get::<gio::File>()
            .unwrap()
            .path()
            .unwrap()
    }

    pub fn note_list(&self) -> NoteList {
        let imp = imp::NoteManager::from_instance(self);
        imp.note_list
            .get()
            .expect("Please call `load notes` first")
            .clone()
    }

    pub async fn load_notes(&self) -> Result<()> {
        let files = fs::read_dir(self.path())?;
        let note_list = NoteList::new();

        for file in files.flatten() {
            let file_path = file.path();

            log::info!("Loading file: {}", file_path.display());

            // TODO consider using sourcefile here
            let file = gio::File::for_path(file_path);
            let note = Note::deserialize(&file).await?;
            note_list.append(note);
        }

        self.set_property("note-list", note_list).unwrap();

        Ok(())
    }

    pub async fn save_note(&self, note: Note) -> Result<()> {
        if note.is_saved() {
            log::info!("Note is already saved returning");
            return Ok(());
        }

        let note_bytes = note.serialize()?;

        note.file()
            .replace_contents_async_future(note_bytes, None, false, gio::FileCreateFlags::NONE)
            .await
            .unwrap();

        note.set_is_saved(true);

        log::info!(
            "Saved note with title of {} and path of {:?}",
            note.metadata().title(),
            note.file().path().unwrap().display()
        );

        Ok(())
    }

    pub fn save_all_notes(&self) -> Result<()> {
        for note in self.note_list().iter() {
            if note.is_saved() {
                log::info!("Note already saved, skipping...");
                continue;
            }

            let note_bytes = note.serialize()?;

            note.file().replace_contents(
                &note_bytes,
                None,
                false,
                gio::FileCreateFlags::NONE,
                None::<&gio::Cancellable>,
            )?;

            note.set_is_saved(true);

            log::info!(
                "Saved note synchronously with title of {} and path of {:?}",
                note.metadata().title(),
                note.file().path().unwrap().display()
            );
        }

        Ok(())
    }

    pub fn create_note(&self) -> Result<()> {
        let mut file_path = self.path();
        file_path.push(Self::generate_unique_file_name());
        file_path.set_extension("md");

        let file = gio::File::for_path(file_path.display().to_string());
        let new_note = Note::create_default(&file);

        self.note_list().append(new_note);

        log::info!("Created note {}", file_path.display());

        Ok(())
    }

    pub fn delete_note(&self, index: usize) -> Result<()> {
        let note_list = self.note_list();
        note_list.remove(index);

        let note = note_list
            .item(index as u32)
            .unwrap()
            .downcast::<Note>()
            .unwrap();

        note.delete().unwrap();

        log::info!("Deleted note {}", note.file().path().unwrap().display());

        Ok(())
    }

    fn generate_unique_file_name() -> String {
        chrono::Local::now()
            .format("Noteworthy-%Y-%m-%d-%H-%M-%S-%f")
            .to_string()
    }
}
