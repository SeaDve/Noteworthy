use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Serialize, Serializer};

use std::cell::{Cell, RefCell};

use crate::date::Date;

mod imp {
    use super::*;

    #[derive(Debug, Default, Serialize)]
    pub struct Metadata {
        pub title: RefCell<String>,
        pub last_modified: RefCell<Date>,
        pub is_pinned: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Metadata {
        const NAME: &'static str = "NwtyNoteMetadata";
        type Type = super::Metadata;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Metadata {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "title",
                        "Title",
                        "Title of the Metadata",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boxed(
                        "last-modified",
                        "Last Modified",
                        "Last modified date of the Metadata",
                        Date::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-pinned",
                        "Is Pinned",
                        "Whether the note associated with self is pinned",
                        false,
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
                "title" => {
                    let title = value.get().unwrap();
                    self.title.replace(title);

                    obj.update_last_modified();
                }
                "last-modified" => {
                    let last_modified = value.get().unwrap();
                    self.last_modified.replace(last_modified);
                }
                "is-pinned" => {
                    let is_pinned = value.get().unwrap();
                    self.is_pinned.set(is_pinned);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "last-modified" => self.last_modified.borrow().to_value(),
                "is-pinned" => self.is_pinned.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Metadata(ObjectSubclass<imp::Metadata>);
}

impl Metadata {
    pub fn new(title: String, last_modified: Date, is_pinned: bool) -> Self {
        glib::Object::new::<Self>(&[
            ("title", &title),
            ("last-modified", &last_modified),
            ("is-pinned", &is_pinned),
        ])
        .expect("Failed to create Metadata.")
    }

    pub fn set_title(&self, title: String) {
        self.set_property("title", title).unwrap();
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }

    pub fn update_last_modified(&self) {
        self.set_property("last-modified", Date::now()).unwrap();
    }

    pub fn last_modified(&self) -> Date {
        self.property("last-modified").unwrap().get().unwrap()
    }

    pub fn set_is_pinned(&self, is_pinned: bool) {
        self.set_property("is-pinned", is_pinned).unwrap();
    }

    pub fn is_pinned(&self) -> bool {
        self.property("is-pinned").unwrap().get().unwrap()
    }
}

impl Serialize for Metadata {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::Metadata::from_instance(self);
        imp.serialize(serializer)
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new(Default::default(), Default::default(), Default::default())
    }
}
