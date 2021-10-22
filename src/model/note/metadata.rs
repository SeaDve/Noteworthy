use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

use crate::model::{Date, NoteTagList};

mod imp {
    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(default)]
    pub struct MetadataInner {
        pub title: String,
        pub tag_list: NoteTagList,
        pub last_modified: Date,
        pub is_pinned: bool,
        pub is_trashed: bool,
    }

    #[derive(Debug, Default)]
    pub struct Metadata {
        pub inner: RefCell<MetadataInner>,
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
                        "Title of the note",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "tag-list",
                        "Tag List",
                        "List containing the tags of the note",
                        NoteTagList::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boxed(
                        "last-modified",
                        "Last Modified",
                        "Last modified date of the note",
                        Date::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-pinned",
                        "Is Pinned",
                        "Whether the note is pinned",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-trashed",
                        "Is Trashed",
                        "Whether the note is in trash",
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

                    if title == obj.title() {
                        return;
                    }

                    obj.update_last_modified();
                    self.inner.borrow_mut().title = title;
                    obj.notify("title");
                }
                "tag-list" => {
                    let tag_list = value.get().unwrap();
                    self.inner.borrow_mut().tag_list = tag_list;
                }
                "last-modified" => {
                    let last_modified = value.get().unwrap();
                    self.inner.borrow_mut().last_modified = last_modified;
                }
                "is-pinned" => {
                    let is_pinned = value.get().unwrap();
                    self.inner.borrow_mut().is_pinned = is_pinned;
                }
                "is-trashed" => {
                    let is_trashed = value.get().unwrap();
                    self.inner.borrow_mut().is_trashed = is_trashed;
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.inner.borrow().title.to_value(),
                "tag-list" => self.inner.borrow().tag_list.to_value(),
                "last-modified" => self.inner.borrow().last_modified.to_value(),
                "is-pinned" => self.inner.borrow().is_pinned.to_value(),
                "is-trashed" => self.inner.borrow().is_trashed.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Metadata(ObjectSubclass<imp::Metadata>);
}

impl Metadata {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).expect("Failed to create Metadata.")
    }

    pub fn set_title(&self, title: &str) {
        self.set_property("title", title).unwrap();
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }

    pub fn set_tag_list(&self, tag_list: NoteTagList) {
        self.set_property("tag-list", tag_list).unwrap();
    }

    pub fn tag_list(&self) -> NoteTagList {
        self.property("tag-list").unwrap().get().unwrap()
    }

    pub fn set_last_modified(&self, date: &Date) {
        self.set_property("last-modified", date).unwrap();
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

    pub fn set_is_trashed(&self, is_trashed: bool) {
        self.set_property("is-trashed", is_trashed).unwrap();
    }

    pub fn is_trashed(&self) -> bool {
        self.property("is-trashed").unwrap().get().unwrap()
    }

    pub fn update_last_modified(&self) {
        self.set_last_modified(&Date::now());
    }

    pub fn update(&self, other: &Metadata) {
        self.set_title(&other.title());
        self.set_tag_list(other.tag_list());
        self.set_last_modified(&other.last_modified());
        self.set_is_pinned(other.is_pinned());
        self.set_is_trashed(other.is_trashed());
    }
}

impl Serialize for Metadata {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::Metadata::from_instance(self);
        imp.inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = imp::MetadataInner::deserialize(deserializer)?;

        let metadata = Self::new();
        let imp = imp::Metadata::from_instance(&metadata);
        imp.inner.replace(inner);

        Ok(metadata)
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::Tag;

    #[test]
    fn title() {
        let metadata = Metadata::new();
        assert_eq!(metadata.title(), "");
        metadata.set_title("A Title");
        assert_eq!(metadata.title(), "A Title");
    }

    #[test]
    fn title_did_not_changed() {
        let metadata = Metadata::new();
        metadata.set_title("Title");
        let old_last_modified = metadata.last_modified();
        metadata.set_title("Title");
        let new_last_modified = metadata.last_modified();
        assert_eq!(old_last_modified, new_last_modified);
    }

    #[test]
    fn title_did_changed() {
        let metadata = Metadata::new();
        metadata.set_title("Title");
        let old_last_modified = metadata.last_modified();
        metadata.set_title("New Title");
        let new_last_modified = metadata.last_modified();
        assert!(old_last_modified < new_last_modified);
    }

    #[test]
    fn tag_list() {
        let metadata = Metadata::new();
        assert!(metadata.tag_list().is_empty());

        let new_tag_list = NoteTagList::new();
        new_tag_list.append(Tag::new("A Tag")).unwrap();

        metadata.set_tag_list(new_tag_list.clone());
        assert!(!metadata.tag_list().is_empty());
        assert_eq!(metadata.tag_list(), new_tag_list);
    }

    #[test]
    fn last_modified() {
        let metadata = Metadata::new();
        assert_eq!(metadata.title(), "");
        metadata.set_title("A Title");
        assert_eq!(metadata.title(), "A Title");
    }

    #[test]
    fn update_last_modified() {
        let metadata = Metadata::new();
        let old_last_modified = metadata.last_modified();
        metadata.update_last_modified();
        assert!(old_last_modified < metadata.last_modified());
    }

    #[test]
    fn is_pinned() {
        let metadata = Metadata::new();
        assert!(!metadata.is_pinned());
        metadata.set_is_pinned(true);
        assert!(metadata.is_pinned());
    }

    #[test]
    fn is_trashed() {
        let metadata = Metadata::new();
        assert!(!metadata.is_trashed());
        metadata.set_is_trashed(true);
        assert!(metadata.is_trashed());
    }

    #[test]
    fn update() {
        let metadata = Metadata::new();
        assert_eq!(metadata.title(), "");
        assert!(metadata.tag_list().is_empty());
        assert!(!metadata.is_pinned());
        assert!(!metadata.is_trashed());

        let other_metadata = Metadata::new();
        other_metadata.set_title("Title");

        let tag_list = NoteTagList::new();
        tag_list.append(Tag::new("A Tag")).unwrap();

        other_metadata.set_tag_list(tag_list);
        other_metadata.set_last_modified(&Date::now());
        other_metadata.set_is_pinned(true);
        other_metadata.set_is_trashed(true);

        metadata.update(&other_metadata);
        assert_eq!(metadata.title(), other_metadata.title());
        assert!(!metadata.tag_list().is_empty());
        assert_eq!(metadata.tag_list(), other_metadata.tag_list());
        assert_eq!(metadata.last_modified(), other_metadata.last_modified());
        assert_eq!(metadata.is_pinned(), other_metadata.is_pinned());
        assert_eq!(metadata.is_trashed(), other_metadata.is_trashed());
    }
}
