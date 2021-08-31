use gtk::{glib, prelude::*, subclass::prelude::*};

use super::{Note, NoteImpl};

use std::{cell::RefCell, path::Path};

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
        fn replace_title(&self, parent: &Self::ParentType, title: &str) {
            let obj: Self::Type = parent.clone().downcast().unwrap();
        }

        fn retrieve_title(&self, parent: &Self::ParentType) -> String {
            let obj: Self::Type = parent.clone().downcast().unwrap();
            let path = obj.path();

            let path = Path::new(&path);
            path.file_name().unwrap().to_string_lossy().to_string()
        }

        fn replace_content(&self, parent: &Self::ParentType, content: &str) {
            let obj: Self::Type = parent.clone().downcast().unwrap();
        }

        fn retrieve_content(&self, parent: &Self::ParentType) -> String {
            let obj: Self::Type = parent.clone().downcast().unwrap();
            let path = obj.path();

            use std::io::Read;
            let mut f = std::fs::File::open(path).expect("file not found");

            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .expect("something went wrong reading the file");

            contents
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
