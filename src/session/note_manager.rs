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
                        glib::ParamFlags::READABLE,
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
            .get_or_init(|| self.retrive_notes().unwrap())
            .clone()
    }

    fn retrive_notes(&self) -> Result<NoteList> {
        let paths = fs::read_dir(self.path())?;
        let note_list = NoteList::new();

        for path in paths.flatten() {
            let path = path.path();

            log::info!("Loading file: {}", path.display());

            // TODO consider using sourcefile here
            let file = gio::File::for_path(path);
            let note = Note::from_file(&file);
            note_list.append(note);
        }

        Ok(note_list)
    }

    pub async fn save_note(&self, note: Note) -> Result<()> {
        if note.is_saved() {
            log::info!("Note is already saved returning");
            return Ok(());
        }

        let note_bytes = note.serialize()?;
        let note_file = note.file();

        note_file
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
        let note_list = self.note_list();

        // FIXME use iterator here
        for i in 0..note_list.n_items() {
            let note = note_list.item(i).unwrap().downcast::<Note>().unwrap();

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
        file_path.push(self.generate_unique_file_name());
        file_path.set_extension("md");

        let file = gio::File::for_path(file_path.display().to_string());
        file.create(gio::FileCreateFlags::NONE, None::<&gio::Cancellable>)?;
        let new_note = Note::from_file(&file);

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

    fn generate_unique_file_name(&self) -> String {
        chrono::Local::now()
            .format("Noteworthy %f %Y%m%dT%H%M%S")
            .to_string()
    }
}
