use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{cell::RefCell, path::PathBuf};

use once_cell::unsync::OnceCell;

use crate::core::{DateTime, FileType};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(default)]
    pub struct AttachmentInner {
        #[serde(serialize_with = "serialize_file")]
        #[serde(deserialize_with = "deserialize_file")]
        pub file: gio::File,
        pub created: DateTime,
        pub title: String,
    }

    impl Default for AttachmentInner {
        fn default() -> Self {
            Self {
                file: gio::File::for_path(glib::tmp_dir()),
                created: DateTime::default(),
                title: String::default(),
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct Attachment {
        pub inner: RefCell<AttachmentInner>,
        pub file_type: OnceCell<FileType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Attachment {
        const NAME: &'static str = "NwtyAttachment";
        type Type = super::Attachment;
    }

    impl ObjectImpl for Attachment {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "file",
                        "File",
                        "File representing where the attachment is stored",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "created",
                        "Created",
                        "The date when the attachment is created",
                        DateTime::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecString::new(
                        "title",
                        "Title",
                        "Title of the attachment",
                        None,
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
                    self.inner.borrow_mut().file = file;
                }
                "created" => {
                    let created = value.get().unwrap();
                    self.inner.borrow_mut().created = created;
                }
                "title" => {
                    let title = value.get().unwrap();
                    self.inner.borrow_mut().title = title;
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.inner.borrow().file.to_value(),
                "created" => self.inner.borrow().created.to_value(),
                "title" => self.inner.borrow().title.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Attachment(ObjectSubclass<imp::Attachment>);
}

impl Attachment {
    pub fn new(file: &gio::File, created: &DateTime) -> Self {
        glib::Object::new::<Self>(&[("file", file), ("created", created)])
            .expect("Failed to create Attachment.")
    }

    // TODO maybe include this to the serialized data, so just deserialize it when loading
    pub fn file_type(&self) -> FileType {
        let file_type = self
            .imp()
            .file_type
            .get_or_init(|| FileType::for_file(&self.file()));

        *file_type
    }

    pub fn file(&self) -> gio::File {
        self.property("file")
    }

    pub fn created(&self) -> DateTime {
        self.property("created")
    }

    pub fn title(&self) -> String {
        self.property("title")
    }

    pub fn set_title(&self, title: &str) {
        self.set_property("title", title);
    }

    pub fn connect_title_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("title"), move |obj, _| f(obj))
    }

    pub async fn delete(&self) {
        let file = self.file();

        if let Err(err) = file.delete_future(glib::PRIORITY_DEFAULT_IDLE).await {
            log::error!("Failed to delete attachment: {:?}", err);
        } else {
            log::info!("Successfully deleted attachment at `{}`", file.uri());
        }
    }
}

impl Default for Attachment {
    fn default() -> Self {
        Self::new(&gio::File::for_path(glib::tmp_dir()), &DateTime::default())
    }
}

// TODO add way for subclasses to include data here
// It is helpful for caching the duration of an audio or save the peaks to show visualization later
impl Serialize for Attachment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.imp().inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Attachment {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = imp::AttachmentInner::deserialize(deserializer)?;

        let attachment = Self::new(&inner.file, &inner.created);
        attachment.set_title(&inner.title);

        Ok(attachment)
    }
}

pub fn serialize_file<S>(file: &gio::File, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // TODO Use relative path instead of absolute path
    // This is to lessen redundancy
    // I think this should be handled somewhere in Note or AttachmentList
    file.path().unwrap().serialize(s)
}

pub fn deserialize_file<'de, D>(deserializer: D) -> Result<gio::File, D::Error>
where
    D: Deserializer<'de>,
{
    let path = PathBuf::deserialize(deserializer)?;

    Ok(gio::File::for_path(path))
}
