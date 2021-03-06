use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct Tag {
        pub name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Tag {
        const NAME: &'static str = "NwtyTag";
        type Type = super::Tag;
    }

    impl ObjectImpl for Tag {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::new(
                    "name",
                    "Name",
                    "Name of the tag",
                    None,
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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
                "name" => {
                    let name = value.get().unwrap();
                    obj.set_name(name);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "name" => obj.name().to_value(),
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
        glib::Object::new(&[("name", &name.to_string())]).expect("Failed to create Tag.")
    }

    /// Must not be called directly if a tag is in a `TagList` or `NoteTagList`.
    /// Use `TagList::rename_tag` instead as it contains sanity checks and other handling.
    pub(super) fn set_name(&self, name: &str) {
        self.imp().name.replace(name.to_string());
        self.notify("name");
    }

    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }

    pub fn connect_name_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("name"), move |obj, _| f(obj))
    }
}

impl Serialize for Tag {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.imp().name.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        Ok(Self::new(&name))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn name() {
        let tag = Tag::new("Tag 1");
        assert_eq!(tag.name(), "Tag 1");

        tag.set_name("New name");
        assert_eq!(tag.name(), "New name");
    }

    #[test]
    fn serialize() {
        let tag = Tag::new("A tag");
        assert_eq!(serde_yaml::to_string(&tag).unwrap(), "---\nA tag\n");
    }

    #[test]
    fn deserialize() {
        let tag: Tag = serde_yaml::from_str("A tag").unwrap();
        assert_eq!(tag.name(), "A tag");
    }
}
