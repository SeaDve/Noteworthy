use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use std::cell::RefCell;

use super::{Note, NoteImpl};
use crate::Result;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LocalNote {
        file: RefCell<Option<gio::File>>,
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
                vec![glib::ParamSpec::new_object(
                    "file",
                    "File",
                    "File representing where the note is stored",
                    gio::File::static_type(),
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
                "file" => {
                    let file = value.get().unwrap();
                    self.file.replace(file);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.file.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    // FIXME Bad hack, make note as an interface instead
    impl NoteImpl for LocalNote {
        fn replace_title(&self, parent: &Self::ParentType, title: &str) -> Result<()> {
            let obj: &Self::Type = parent.downcast_ref().unwrap();

            Ok(())
        }

        fn retrieve_title(&self, parent: &Self::ParentType) -> Result<String> {
            let obj: &Self::Type = parent.downcast_ref().unwrap();
            let file = obj.file();

            Ok(file.basename().unwrap().display().to_string())
        }

        fn replace_content(&self, parent: &Self::ParentType, content: &str) -> Result<()> {
            let obj: &Self::Type = parent.downcast_ref().unwrap();
            let file = obj.file();

            file.replace_contents(
                content.as_bytes(),
                None,
                false,
                gio::FileCreateFlags::NONE,
                None::<&gio::Cancellable>,
            )?;

            log::info!("Replaced contents");

            Ok(())
        }

        fn retrieve_content(&self, parent: &Self::ParentType) -> Result<String> {
            let obj: &Self::Type = parent.downcast_ref().unwrap();
            let file = obj.file();

            let (contents, _) = file.load_contents(None::<&gio::Cancellable>)?;
            let contents = String::from_utf8(contents)?;

            Ok(contents)
        }
    }
}

glib::wrapper! {
    pub struct LocalNote(ObjectSubclass<imp::LocalNote>)
        @extends Note;
}

impl LocalNote {
    pub fn load_from_file(file: &gio::File) -> Result<Self> {
        Ok(glib::Object::new::<Self>(&[("file", file)]).expect("Failed to create LocalNote."))
    }

    pub fn from_file(file: &gio::File) -> Result<Self> {
        file.create(gio::FileCreateFlags::NONE, None::<&gio::Cancellable>)?;

        Ok(glib::Object::new::<Self>(&[("file", file)]).expect("Failed to create LocalNote."))
    }

    pub fn file(&self) -> gio::File {
        self.property("file")
            .unwrap()
            .get::<Option<gio::File>>()
            .unwrap()
            .unwrap()
    }

    pub fn set_file(&self, file: &gio::File) {
        self.set_property("file", Some(file)).unwrap();
    }

    pub fn delete(&self) -> Result<()> {
        self.file().delete(None::<&gio::Cancellable>)?;
        Ok(())
    }
}
