use gray_matter::{engine::YAML, Matter};
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use std::cell::Cell;

use super::{NoteId, NoteMetadata};

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct Note {
        pub file: OnceCell<gio::File>,
        pub metadata: OnceCell<NoteMetadata>,
        pub buffer: OnceCell<gtk_source::Buffer>,
        pub is_saved: Cell<bool>,
        pub id: OnceCell<NoteId>,
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
                        "File where Self is stored",
                        gio::File::static_type(),
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "metadata",
                        "Metadata",
                        "Contains information about Self",
                        NoteMetadata::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "buffer",
                        "Buffer",
                        "Contains content fo Self",
                        gtk_source::Buffer::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-saved",
                        "Is Saved",
                        "Whether the content is saved to file",
                        false,
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => obj.file().to_value(),
                "metadata" => obj.metadata().to_value(),
                "is-saved" => obj.is_saved().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_signals();
            obj.set_is_saved(true);
        }
    }
}

glib::wrapper! {
    pub struct Note(ObjectSubclass<imp::Note>);
}

impl Note {
    pub fn new(file: &gio::File) -> Self {
        glib::Object::new(&[
            ("file", &file),
            ("metadata", &NoteMetadata::default()),
            ("buffer", &Self::default_buffer()),
        ])
        .expect("Failed to create Note.")
    }

    pub async fn load(file: &gio::File) -> anyhow::Result<Self> {
        let (metadata, content) = Self::load_metadata_and_content(file).await?;

        let buffer = Self::default_buffer();
        buffer.set_text(&content);

        Ok(glib::Object::new(&[
            ("file", &file),
            ("metadata", &metadata),
            ("buffer", &buffer),
        ])
        .expect("Failed to create Note."))
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        if self.is_saved() {
            log::warn!("Note is already saved, trying to save again");
            return Ok(());
        }

        // FIXME replace with non hacky implementation
        let mut bytes = serde_yaml::to_vec(&self.metadata())?;

        let delimiter = "---\n";
        bytes.append(&mut delimiter.into());

        let buffer = self.buffer();
        let (start_iter, end_iter) = buffer.bounds();
        let buffer_text = buffer.text(&start_iter, &end_iter, true).to_string();
        bytes.append(&mut buffer_text.into_bytes());

        self.file()
            .replace_contents_future(bytes, None, false, gio::FileCreateFlags::NONE)
            .await
            .map_err(|err| err.1)?;

        self.set_is_saved(true);

        log::info!("Saved `{}`", self);

        Ok(())
    }

    pub fn metadata(&self) -> &NoteMetadata {
        self.imp().metadata.get().unwrap()
    }

    pub fn buffer(&self) -> &gtk_source::Buffer {
        self.imp().buffer.get().unwrap()
    }

    pub fn id(&self) -> &NoteId {
        self.imp()
            .id
            .get_or_init(|| NoteId::from_path(&self.file().path().unwrap()))
    }

    pub fn is_saved(&self) -> bool {
        self.imp().is_saved.get()
    }

    pub fn connect_is_saved_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("is-saved"), move |obj, _| f(obj))
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

    pub async fn update(&self) -> anyhow::Result<()> {
        let (metadata, content) = Self::load_metadata_and_content(self.file()).await?;

        self.metadata().update(&metadata);
        self.buffer().set_text(&content);

        Ok(())
    }

    fn set_is_saved(&self, is_saved: bool) {
        self.imp().is_saved.set(is_saved);
        self.notify("is-saved");
    }

    fn file(&self) -> &gio::File {
        self.imp().file.get().unwrap()
    }

    async fn load_metadata_and_content(file: &gio::File) -> anyhow::Result<(NoteMetadata, String)> {
        let (file_content, _) = file.load_contents_future().await?;
        let file_content = std::str::from_utf8(&file_content)?;
        let parsed_entity = Matter::<YAML>::new().parse(file_content);

        let metadata: NoteMetadata = parsed_entity
            .data
            .and_then(|p| {
                p.deserialize()
                    .map_err(|err| log::warn!("Failed to deserialize data: {:?}", err))
                    .ok()
            })
            .unwrap_or_default();

        Ok((metadata, parsed_entity.content))
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

    fn setup_signals(&self) {
        self.buffer()
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.metadata().update_last_modified();
                obj.set_is_saved(false);
            }));

        let metadata = self.metadata();

        metadata.connect_notify_local(
            None,
            clone!(@weak self as obj => move |_, _| {
                obj.emit_by_name::<()>("metadata-changed", &[]);
                obj.set_is_saved(false);
            }),
        );

        // TODO not sure if we need to notify metadata-changed here (same with attachment_list)
        // Unless we want to show the tags in the sidebar
        metadata
            .tag_list()
            .connect_items_changed(clone!(@weak self as obj => move |_, _, _, _| {
                obj.emit_by_name::<()>("metadata-changed", &[]);
                obj.set_is_saved(false);
            }));

        metadata.attachment_list().connect_items_changed(
            clone!(@weak self as obj => move |_, _, _, _| {
                obj.emit_by_name::<()>("metadata-changed", &[]);
                obj.set_is_saved(false);
            }),
        );
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
