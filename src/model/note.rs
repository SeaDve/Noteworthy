use gray_matter::{engine::YAML, Matter};
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use std::{cell::Cell, path::Path};

use super::{NoteId, NoteMetadata};
use crate::utils;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct Note {
        pub file: OnceCell<gio::File>,
        pub is_saved: Cell<bool>,
        pub metadata: OnceCell<NoteMetadata>,
        pub buffer: OnceCell<gtk_source::Buffer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Note {
        const NAME: &'static str = "NwtyNote";
        type Type = super::Note;
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
                    glib::ParamSpecObject::new(
                        "file",
                        "File",
                        "File representing where the note is stored",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "metadata",
                        "Metadata",
                        "Metadata containing info of note",
                        NoteMetadata::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "buffer",
                        "Buffer",
                        "The buffer containing note text content",
                        gtk_source::Buffer::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoolean::new(
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
                    obj.emit_by_name::<()>("metadata-changed", &[]);
                    obj.set_is_saved(false);
                }),
            );

            // TODO not sure if we need to notify metadata-changed here (same with attachment_list)
            // Unless we want to show the tags in the sidebar
            metadata
                .tag_list()
                .connect_items_changed(clone!(@weak obj => move |_, _, _, _| {
                    obj.emit_by_name::<()>("metadata-changed", &[]);
                    obj.set_is_saved(false);
                }));

            metadata.attachment_list().connect_items_changed(
                clone!(@weak obj => move |_, _, _, _| {
                    obj.emit_by_name::<()>("metadata-changed", &[]);
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
    pub fn new(file: &gio::File, metadata: &NoteMetadata, buffer: &gtk_source::Buffer) -> Self {
        glib::Object::new::<Self>(&[("file", file), ("metadata", metadata), ("buffer", buffer)])
            .expect("Failed to create Note.")
    }

    pub fn create_default(base_path: impl AsRef<Path>) -> Self {
        let file_path = utils::generate_unique_path(base_path, "Note", Some("md"));
        let file = gio::File::for_path(&file_path);

        Self::new(&file, &NoteMetadata::default(), &Self::default_buffer())
    }

    pub fn file(&self) -> gio::File {
        self.imp().file.get().unwrap().clone()
    }

    pub fn metadata(&self) -> NoteMetadata {
        self.imp().metadata.get().unwrap().clone()
    }

    pub fn buffer(&self) -> gtk_source::Buffer {
        self.imp().buffer.get().unwrap().clone()
    }

    pub fn id(&self) -> NoteId {
        NoteId::from_path(&self.file().path().unwrap())
    }

    pub fn is_saved(&self) -> bool {
        self.imp().is_saved.get()
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
    }

    pub fn connect_is_saved_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("is-saved"), move |obj, _| f(obj))
    }

    pub async fn update(&self) -> anyhow::Result<()> {
        let (file_content, _) = self.file().load_contents_future().await?;
        let file_content = std::str::from_utf8(&file_content)?;
        let parsed_entity = Matter::<YAML>::new().parse(file_content);

        let new_metadata: NoteMetadata = parsed_entity
            .data
            .and_then(|p| p.deserialize().ok())
            .unwrap_or_default();

        let imp = self.imp();

        let metadata = imp.metadata.get().unwrap();
        metadata.update(&new_metadata);

        let buffer = imp.buffer.get().unwrap();
        buffer.set_text(&parsed_entity.content);

        Ok(())
    }

    pub async fn deserialize(file: &gio::File) -> anyhow::Result<Self> {
        let (file_content, _) = file.load_contents_future().await?;
        let file_content = std::str::from_utf8(&file_content)?;
        let parsed_entity = Matter::<YAML>::new().parse(file_content);

        let metadata: NoteMetadata = parsed_entity
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
            .replace_contents_future(bytes, None, false, gio::FileCreateFlags::NONE)
            .await
            .map_err(|err| err.1)?;

        self.set_is_saved(true);

        log::info!("Saved `{}`", self);

        Ok(())
    }

    fn set_is_saved(&self, is_saved: bool) {
        self.set_property("is-saved", is_saved);
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
        gtk_source::Buffer::builder()
            .highlight_matching_brackets(false)
            .language(
                &gtk_source::LanguageManager::default()
                    .language("markdown")
                    .unwrap(),
            )
            .build()
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Note at path `{}` with title `{}`",
            self.file().path().unwrap().display(),
            self.metadata().title()
        )
    }
}
