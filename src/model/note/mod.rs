mod metadata;

use gray_matter::{engine::YAML, Matter};
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use std::{cell::Cell, path::Path};

use self::metadata::Metadata;
use super::NoteId;
use crate::utils;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct Note {
        pub file: OnceCell<gio::File>,
        pub is_saved: Cell<bool>,
        pub metadata: OnceCell<Metadata>,
        pub buffer: OnceCell<gtk_source::Buffer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Note {
        const NAME: &'static str = "NwtyNote";
        type Type = super::Note;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Note {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("metadata-changed", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
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
                        "buffer",
                        "Buffer",
                        "The buffer containing note text content",
                        gtk_source::Buffer::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-saved",
                        "Is Saved",
                        "Whether the note is already saved to file",
                        false,
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
                "file" => {
                    let file = value.get().unwrap();
                    self.file.set(file).unwrap();
                }
                "metadata" => {
                    let metadata = value.get().unwrap();
                    self.metadata.set(metadata).unwrap();
                }
                "buffer" => {
                    let buffer = value.get().unwrap();
                    self.buffer.set(buffer).unwrap();
                }
                "is-saved" => {
                    let is_saved = value.get().unwrap();
                    self.is_saved.set(is_saved);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => obj.file().to_value(),
                "metadata" => obj.metadata().to_value(),
                "buffer" => obj.buffer().to_value(),
                "is-saved" => obj.is_saved().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_is_saved(true);

            let metadata = obj.metadata();

            obj.buffer().connect_changed(clone!(@weak obj => move |_| {
                obj.metadata().update_last_modified();
                obj.notify("buffer"); // For some reason the subtitle doesn't get updated when the filter model is not incremental
                obj.set_is_saved(false);
            }));

            metadata.connect_notify_local(
                None,
                clone!(@weak obj => move |_, _| {
                    obj.emit_by_name("metadata-changed", &[]).unwrap();
                    obj.set_is_saved(false);
                }),
            );

            // TODO not sure if we need to notify metadata-changed here (same with attachment_list)
            // Unless we want to show the tags in the sidebar
            metadata
                .tag_list()
                .connect_items_changed(clone!(@weak obj => move |_, _, _, _| {
                    obj.emit_by_name("metadata-changed", &[]).unwrap();
                    obj.set_is_saved(false);
                }));

            metadata.attachment_list().connect_items_changed(
                clone!(@weak obj => move |_, _, _, _| {
                    obj.emit_by_name("metadata-changed", &[]).unwrap();
                    obj.set_is_saved(false);
                }),
            );
        }
    }
}

glib::wrapper! {
    pub struct Note(ObjectSubclass<imp::Note>);
}

impl Note {
    pub fn new(file: &gio::File, metadata: &Metadata, buffer: &gtk_source::Buffer) -> Self {
        glib::Object::new::<Self>(&[("file", file), ("metadata", metadata), ("buffer", buffer)])
            .expect("Failed to create Note.")
    }

    pub fn create_default(base_path: &Path) -> Self {
        let file = {
            let file_name = utils::generate_unique_file_name("Note"); // Also the note's Id
            let mut file_path = base_path.join(file_name);
            file_path.set_extension("md");
            gio::File::for_path(file_path)
        };

        Self::new(&file, &Metadata::default(), &Self::default_buffer())
    }

    pub fn file(&self) -> gio::File {
        let imp = imp::Note::from_instance(self);
        imp.file.get().unwrap().clone()
    }

    pub fn metadata(&self) -> Metadata {
        let imp = imp::Note::from_instance(self);
        imp.metadata.get().unwrap().clone()
    }

    pub fn buffer(&self) -> gtk_source::Buffer {
        let imp = imp::Note::from_instance(self);
        imp.buffer.get().unwrap().clone()
    }

    pub fn id(&self) -> NoteId {
        NoteId::from_path(&self.file().path().unwrap())
    }

    pub fn is_saved(&self) -> bool {
        let imp = imp::Note::from_instance(self);
        imp.is_saved.get()
    }

    fn set_is_saved(&self, is_saved: bool) {
        self.set_property("is-saved", is_saved).unwrap();
    }

    pub fn delete(&self) -> anyhow::Result<()> {
        self.file().delete(None::<&gio::Cancellable>)?;
        Ok(())
    }

    pub fn connect_metadata_changed<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_local("metadata-changed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
        .unwrap()
    }

    pub fn connect_is_saved_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("is-saved"), move |obj, _| f(obj))
    }

    pub async fn update(&self) -> anyhow::Result<()> {
        let (file_content, _) = self.file().load_contents_async_future().await?;
        let file_content = std::str::from_utf8(&file_content)?;
        let parsed_entity = Matter::<YAML>::new().parse(file_content);

        let new_metadata: Metadata = parsed_entity
            .data
            .and_then(|p| p.deserialize().ok())
            .unwrap_or_default();

        let imp = imp::Note::from_instance(self);

        let metadata = imp.metadata.get().unwrap();
        metadata.update(&new_metadata);

        let buffer = imp.buffer.get().unwrap();
        buffer.set_text(&parsed_entity.content);

        Ok(())
    }

    pub async fn deserialize(file: &gio::File) -> anyhow::Result<Self> {
        let (file_content, _) = file.load_contents_async_future().await?;
        let file_content = std::str::from_utf8(&file_content)?;
        let parsed_entity = Matter::<YAML>::new().parse(file_content);

        let metadata: Metadata = parsed_entity
            .data
            .and_then(|p| p.deserialize().ok())
            .unwrap_or_default();

        let buffer = Self::default_buffer();
        buffer.set_text(&parsed_entity.content);

        log::info!("File `{}` is loaded", file.path().unwrap().display());

        Ok(Self::new(file, &metadata, &buffer))
    }

    pub async fn serialize(&self) -> anyhow::Result<()> {
        if self.is_saved() {
            // TODO consider removing this
            log::error!("Note is already saved, trying to save again");
            return Ok(());
        }

        let bytes = self.serialize_to_bytes()?;
        self.file()
            .replace_contents_async_future(bytes, None, false, gio::FileCreateFlags::NONE)
            .await
            .map_err(|err| err.1)?;

        self.set_is_saved(true);

        log::info!(
            "Saved noted with title of `{}` and path of `{}`",
            self.metadata().title(),
            self.file().path().unwrap().display()
        );

        Ok(())
    }

    fn serialize_to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        // FIXME replace with not hacky implementation
        let mut bytes = serde_yaml::to_vec(&self.metadata())?;

        let delimiter = "---\n".to_string();
        bytes.append(&mut delimiter.into_bytes());

        let buffer = self.buffer();
        let (start_iter, end_iter) = buffer.bounds();
        let buffer_text = buffer.text(&start_iter, &end_iter, true).to_string();
        bytes.append(&mut buffer_text.into_bytes());

        Ok(bytes)
    }

    fn default_buffer() -> gtk_source::Buffer {
        // FIXME not following AdwStyleManager::is-dark
        gtk_source::BufferBuilder::new()
            .highlight_matching_brackets(false)
            .language(
                &gtk_source::LanguageManager::default()
                    .and_then(|lm| lm.language("markdown"))
                    .unwrap(),
            )
            .build()
    }
}
