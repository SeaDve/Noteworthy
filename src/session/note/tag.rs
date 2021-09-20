use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Tag {
        pub name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Tag {
        const NAME: &'static str = "NwtyTag";
        type Type = super::Tag;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Tag {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_string(
                    "name",
                    "Name",
                    "Name of the tag",
                    None,
                    glib::ParamFlags::READWRITE,
                )]
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
                "name" => {
                    let name = value.get().unwrap();
                    self.name.replace(name);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => self.name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Tag(ObjectSubclass<imp::Tag>);
}

impl Tag {
    pub fn new(name: &str) -> Self {
        glib::Object::new::<Self>(&[("name", &name.to_string())]).expect("Failed to create Tag.")
    }

    pub fn name(&self) -> String {
        self.property("name").unwrap().get().unwrap()
    }

    pub fn connect_name_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("name"), f)
    }
}

// FIXME better ser & de
impl Serialize for Tag {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::Tag::from_instance(self);
        imp.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let imp = imp::Tag::deserialize(deserializer)?;
        let name = imp.name.borrow();
        Ok(Self::new(&name))
    }
}
