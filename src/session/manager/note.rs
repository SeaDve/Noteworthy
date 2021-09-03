use gray_matter::{engine::YAML, Matter};
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use std::cell::RefCell;

use crate::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub tags: Vec<String>,
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Note {
        pub file: OnceCell<gio::File>,

        pub metadata: RefCell<Option<Metadata>>,
        pub content: RefCell<Option<String>>,
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
                    self.file.set(file).unwrap();
                }
                "title" => {
                    let title = value.get().unwrap();

                    let mut metadata = self.metadata.take().unwrap();
                    metadata.title = title;

                    // FIXME Replace this with not a hacky way
                    let mut metadata_bytes = serde_yaml::to_vec(&metadata).unwrap();
                    metadata_bytes.append(&mut "---\n".as_bytes().to_vec());
                    metadata_bytes.append(&mut obj.content().as_bytes().to_vec());

                    let file = obj.file();
                    file.replace_contents(
                        &metadata_bytes,
                        None,
                        false,
                        gio::FileCreateFlags::NONE,
                        None::<&gio::Cancellable>,
                    )
                    .expect("Failed to create output stream from file");

                    self.metadata.replace(Some(metadata));
                }
                "content" => {
                    let content: Option<String> = value.get().unwrap();

                    // FIXME replace with not hacky implementation
                    let mut metadata_bytes = serde_yaml::to_vec(&self.metadata).unwrap();
                    metadata_bytes.append(&mut "---\n".as_bytes().to_vec());
                    metadata_bytes.append(&mut content.as_ref().unwrap().as_bytes().to_vec());

                    let file = obj.file();
                    file.replace_contents(
                        content.as_ref().unwrap().as_bytes(),
                        None,
                        false,
                        gio::FileCreateFlags::NONE,
                        None::<&gio::Cancellable>,
                    )
                    .expect("Failed to load contents from file");

                    self.content.replace(content);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.file.get().to_value(),
                "title" => self.metadata.borrow().as_ref().unwrap().title.to_value(),
                "content" => self.content.borrow().to_value(),
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
        let obj = glib::Object::new::<Self>(&[("file", file)]).expect("Failed to create Note.");
        obj.load_properties_from_file()?;
        Ok(obj)
    }

    pub fn from_file(file: &gio::File) -> Result<Self> {
        file.create(gio::FileCreateFlags::NONE, None::<&gio::Cancellable>)?;
        let obj = glib::Object::new::<Self>(&[("file", file)]).expect("Failed to create Note.");
        obj.load_properties_from_file()?;
        Ok(obj)
    }

    pub fn set_file(&self, file: &gio::File) {
        self.set_property("file", Some(file)).unwrap();
    }

    pub fn file(&self) -> gio::File {
        self.property("file").unwrap().get::<gio::File>().unwrap()
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

    fn load_properties_from_file(&self) -> Result<()> {
        let file = self.file();
        let (file_content, _) = file.load_contents(None::<&gio::Cancellable>)?;
        let file_content = String::from_utf8(file_content)?;
        let parsed_entity = Matter::<YAML>::new().parse(&file_content);
        let metadata: Metadata = parsed_entity.data.unwrap().deserialize().unwrap();

        let imp = imp::Note::from_instance(self);
        imp.metadata.replace(Some(metadata));
        imp.content.replace(Some(parsed_entity.content));
        Ok(())
    }
}