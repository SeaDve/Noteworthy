use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{cell::RefCell, path::PathBuf};

use super::attachment_kind::AttachmentKind;
use crate::model::DateTime;

mod imp {
    use super::*;

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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Attachment {
        const NAME: &'static str = "NwtyAttachment";
        type Type = super::Attachment;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Attachment {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "file",
                        "File",
                        "File representing where the attachment is stored",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "created",
                        "Created",
                        "The date when the attachment is created",
                        DateTime::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
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

    pub fn kind(&self) -> AttachmentKind {
        AttachmentKind::for_file(&self.file())
    }

    pub fn file(&self) -> gio::File {
        self.property("file").unwrap().get().unwrap()
    }

    pub fn created(&self) -> DateTime {
        self.property("created").unwrap().get().unwrap()
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }

    pub fn set_title(&self, title: &str) {
        self.set_property("title", title).unwrap();
    }

    pub fn connect_title_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("title"), f)
    }
}

impl Default for Attachment {
    fn default() -> Self {
        Self::new(&gio::File::for_path(glib::tmp_dir()), &DateTime::default())
    }
}

impl Serialize for Attachment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::Attachment::from_instance(self);
        imp.inner.serialize(serializer)
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
    file.path().unwrap().serialize(s)
}

pub fn deserialize_file<'de, D>(deserializer: D) -> Result<gio::File, D::Error>
where
    D: Deserializer<'de>,
{
    let path = PathBuf::deserialize(deserializer)?;

    Ok(gio::File::for_path(path))
}
