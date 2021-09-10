mod metadata;

use gray_matter::{engine::YAML, Matter};
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::sync::OnceCell;

pub use self::metadata::Metadata;
use crate::Result;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Note {
        pub file: OnceCell<gio::File>,
        pub metadata: OnceCell<Metadata>,
        pub content: OnceCell<sourceview::Buffer>,
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

            obj.content().connect_changed(clone!(@weak obj => move |_| {
                obj.metadata().update_last_modified();
            }));
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
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "content",
                        "Content",
                        "Content of the note",
                        sourceview::Buffer::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "file" => {
                    let file = value.get().unwrap();
                    self.file.set(file).unwrap();
                }
                "metadata" => {
                    let metadata = value.get().unwrap();
                    self.metadata.set(metadata).unwrap();
                }
                "content" => {
                    let content = value.get().unwrap();
                    self.content.set(content).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => obj.file().to_value(),
                "metadata" => obj.metadata().to_value(),
                "content" => obj.content().to_value(),
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
        let imp = imp::Note::from_instance(self);
        imp.file.get().unwrap().clone()
    }

    pub fn metadata(&self) -> Metadata {
        let imp = imp::Note::from_instance(self);
        imp.metadata.get().unwrap().clone()
    }

    pub fn content(&self) -> sourceview::Buffer {
        let imp = imp::Note::from_instance(self);
        imp.content.get().unwrap().clone()
    }

    pub fn delete(&self) -> Result<()> {
        self.file().delete(None::<&gio::Cancellable>)?;
        Ok(())
    }

    pub fn deserialize_from_file(file: &gio::File) -> Result<Self> {
        let (file_content, _) = file.load_contents(None::<&gio::Cancellable>)?;
        let file_content = std::str::from_utf8(&file_content)?;
        let parsed_entity = Matter::<YAML>::new().parse(file_content);

        let content = sourceview::BufferBuilder::new()
            .text(&parsed_entity.content)
            .highlight_matching_brackets(false)
            .language(
                &sourceview::LanguageManager::default()
                    .and_then(|lm| lm.language("markdown"))
                    .unwrap(),
            )
            .build();

        let metadata: Metadata = parsed_entity
            .data
            .and_then(|p| p.deserialize().ok())
            .unwrap_or_default();

        let note = glib::Object::new::<Self>(&[
            ("file", file),
            ("metadata", &metadata),
            ("content", &content),
        ])
        .expect("Failed to create Note.");

        Ok(note)
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        // FIXME replace with not hacky implementation
        let mut bytes = serde_yaml::to_vec(&self.metadata())?;
        bytes.append(&mut "---\n".as_bytes().to_vec());

        let buffer = self.content();
        let (start_iter, end_iter) = buffer.bounds();
        let buffer_text = buffer.text(&start_iter, &end_iter, true);

        bytes.append(&mut buffer_text.as_bytes().to_vec());

        Ok(bytes)
    }
}
