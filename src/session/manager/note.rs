use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use std::cell::RefCell;

use crate::Result;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Note {
        file: RefCell<Option<gio::File>>,
        title: RefCell<String>,
        content: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Note {
        const NAME: &'static str = "NwtyNote";
        type Type = super::Note;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Note {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "file",
                        "File",
                        "File representing where the note is stored",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "title",
                        "Title",
                        "Title of the notes",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "content",
                        "Content",
                        "Content of the note",
                        None,
                        glib::ParamFlags::READWRITE,
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
                "file" => {
                    let file = value.get().unwrap();
                    self.file.replace(file);
                }
                "title" => {
                    let title = value.get().unwrap();
                    self.title.replace(title);
                }
                "content" => {
                    let content: String = value.get().unwrap();

                    let file = obj.file();
                    file.replace_contents(
                        content.as_bytes(),
                        None,
                        false,
                        gio::FileCreateFlags::NONE,
                        None::<&gio::Cancellable>,
                    )
                    .expect("Failed to load contents from file");

                    self.content.replace(content);
                    log::info!("Replaced contents");
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.file.borrow().to_value(),
                "title" => {
                    let file = obj.file();
                    let title = file.basename().unwrap().display().to_string();
                    title.to_value()

                    // self.title.borrow().to_value()
                }
                "content" => {
                    let file = obj.file();
                    let (content, _) = file
                        .load_contents(None::<&gio::Cancellable>)
                        .expect("Failed to load contents from file");
                    let content = String::from_utf8(content).expect("Failed to load from bytes");
                    content.to_value()

                    // self.content.borrow().to_value()
                }
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Note(ObjectSubclass<imp::Note>);
}

impl Note {
    pub fn load_from_file(file: &gio::File) -> Result<Self> {
        Ok(glib::Object::new::<Self>(&[("file", file)]).expect("Failed to create Note."))
    }

    pub fn from_file(file: &gio::File) -> Result<Self> {
        file.create(gio::FileCreateFlags::NONE, None::<&gio::Cancellable>)?;

        Ok(glib::Object::new::<Self>(&[("file", file)]).expect("Failed to create Note."))
    }

    pub fn set_file(&self, file: &gio::File) {
        self.set_property("file", Some(file)).unwrap();
    }

    pub fn file(&self) -> gio::File {
        self.property("file")
            .unwrap()
            .get::<Option<gio::File>>()
            .unwrap()
            .unwrap()
    }

    pub fn set_title(&self, title: &str) {
        self.set_property("title", title).unwrap();
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }

    pub fn set_content(&self, content: &str) {
        self.set_property("content", content).unwrap();
    }

    pub fn content(&self) -> String {
        self.property("content").unwrap().get().unwrap()
    }

    pub fn delete(&self) -> Result<()> {
        self.file().delete(None::<&gio::Cancellable>)?;
        Ok(())
    }
}
