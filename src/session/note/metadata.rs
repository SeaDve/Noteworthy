use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Serialize, Serializer};

use std::cell::RefCell;

use crate::date::Date;

mod imp {
    use super::*;

    #[derive(Debug, Default, Serialize)]
    pub struct Metadata {
        pub title: RefCell<String>,
        pub last_modified: RefCell<Date>,
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "last-modified" => self.last_modified.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Metadata(ObjectSubclass<imp::Metadata>);
}

impl Metadata {
    pub fn new(title: String, last_modified: Date) -> Self {
        glib::Object::new::<Self>(&[("title", &title), ("last-modified", &last_modified)])
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

    pub fn connect_last_modified_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        // TODO make this also handle other properties to enabled sorting for title etc.
        self.connect_notify_local(Some("last-modified"), f)
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
        Self::new(Default::default(), Default::default())
    }
}
