mod metadata;

use gray_matter::{engine::YAML, value::pod::Pod, Matter};
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;

use std::{cell::RefCell, collections::HashMap};

pub use self::metadata::Metadata;
use crate::Result;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Note {
        pub file: OnceCell<gio::File>,

        pub metadata: RefCell<Metadata>,
        pub content: RefCell<String>,
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
            obj.deserialize_from_file()
                .expect("Failed to deserialize note from file");
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
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "metadata",
                        "Metadata",
                        "Metadata containing info of note",
                        Metadata::static_type(),
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
                "metadata" => {
                    let metadata = value.get().unwrap();
                    self.metadata.replace(metadata);
                }
                "content" => {
                    let content = value.get().unwrap();
                    self.content.replace(content);

                    obj.metadata().update_modified();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.file.get().to_value(),
                "metadata" => self.metadata.borrow().to_value(),
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
    pub fn from_file(file: &gio::File) -> Self {
        glib::Object::new::<Self>(&[("file", file)]).expect("Failed to create Note.")
    }

    pub fn file(&self) -> gio::File {
        self.property("file").unwrap().get::<gio::File>().unwrap()
    }

    pub fn metadata(&self) -> Metadata {
        self.property("metadata").unwrap().get().unwrap()
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

    fn deserialize_from_file(&self) -> Result<()> {
        let file = self.file();
        let (file_content, _) = file.load_contents(None::<&gio::Cancellable>)?;
        let file_content = std::str::from_utf8(&file_content)?;
        let parsed_entity = Matter::<YAML>::new().parse(file_content);

        let metadata = parsed_entity
            .data
            .map(|p| {
                let parsed_entity_data: HashMap<String, Pod> = p.into();
                Metadata::new(
                    parsed_entity_data
                        .get("title")
                        .map(|t| t.as_string().unwrap())
                        .unwrap_or_default(),
                    parsed_entity_data
                        .get("modified")
                        .map(|t| t.as_string().unwrap().into())
                        .unwrap_or_default(),
                )
            })
            .unwrap_or_default();

        let imp = imp::Note::from_instance(self);
        imp.metadata.replace(metadata);
        imp.content.replace(parsed_entity.content);
        Ok(())
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let imp = imp::Note::from_instance(self);

        // FIXME replace with not hacky implementation
        let mut bytes = serde_yaml::to_vec(&imp.metadata).unwrap();
        bytes.append(&mut "---\n".as_bytes().to_vec());
        bytes.append(&mut imp.content.borrow_mut().as_bytes().to_vec());

        Ok(bytes)
    }
}
