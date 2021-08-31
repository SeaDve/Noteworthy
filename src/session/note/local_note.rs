use gtk::{glib, prelude::*, subclass::prelude::*};

use std::{cell::RefCell, path::Path};

use super::{Note, NoteImpl};
use crate::Result;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LocalNote {
        path: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LocalNote {
        const NAME: &'static str = "NwtyLocalNote";
        type Type = super::LocalNote;
        type ParentType = Note;
    }

    impl ObjectImpl for LocalNote {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_string(
                    "path",
                    "Path",
                    "Path where the note is stored",
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
                "path" => {
                    let path = value.get().unwrap();
                    self.path.replace(path);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "path" => self.path.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl NoteImpl for LocalNote {
        fn replace_title(&self, parent: &Self::ParentType, title: &str) -> Result<()> {
            let obj: Self::Type = parent.clone().downcast().unwrap();

            Ok(())
        }

        fn retrieve_title(&self, parent: &Self::ParentType) -> Result<String> {
            let obj: Self::Type = parent.clone().downcast().unwrap();
            let path = obj.path();

            let path = Path::new(&path);
            let note_file_name = path.file_name().unwrap().to_string_lossy().to_string();

            Ok(note_file_name)
        }

        fn replace_content(&self, parent: &Self::ParentType, content: &str) -> Result<()> {
            let obj: Self::Type = parent.clone().downcast().unwrap();
            let path = obj.path();

            use std::io::Write;

            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;

            f.write_all(content.as_bytes())?;

            Ok(())
        }

        fn retrieve_content(&self, parent: &Self::ParentType) -> Result<String> {
            let obj: Self::Type = parent.clone().downcast().unwrap();
            let path = obj.path();

            use std::io::Read;
            let mut f = std::fs::File::open(path)?;

            let mut contents = String::new();
            f.read_to_string(&mut contents)?;

            Ok(contents)
        }
    }
}

glib::wrapper! {
    pub struct LocalNote(ObjectSubclass<imp::LocalNote>)
        @extends Note;
}

impl LocalNote {
    pub fn new(path: &Path) -> Self {
        glib::Object::new::<Self>(&[("path", &path.display().to_string())])
            .expect("Failed to create LocalNote.")
    }

    pub fn path(&self) -> String {
        self.property("path").unwrap().get().unwrap()
    }

    pub fn set_path(&self, path: String) {
        self.set_property("path", path).unwrap();
    }
}
