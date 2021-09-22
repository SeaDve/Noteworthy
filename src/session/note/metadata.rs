use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::{Cell, RefCell};

use super::note_tag_list::NoteTagList;
use crate::date::Date;

mod imp {
    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(default)]
    pub struct Metadata {
        pub title: RefCell<String>,
        pub tag_list: RefCell<NoteTagList>,
        pub last_modified: RefCell<Date>,
        pub is_pinned: Cell<bool>,
        pub is_trashed: Cell<bool>,
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
                        "Title of the metadata",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "tag-list",
                        "Tag List",
                        "List containing the tags",
                        NoteTagList::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boxed(
                        "last-modified",
                        "Last Modified",
                        "Last modified date of the metadata",
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
                    glib::ParamSpec::new_boolean(
                        "is-trashed",
                        "Is Trashed",
                        "Whether the note associated with self is in trash",
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
                "tag-list" => {
                    let tag_list = value.get().unwrap();
                    self.tag_list.replace(tag_list);
                }
                "last-modified" => {
                    let last_modified = value.get().unwrap();
                    self.last_modified.replace(last_modified);
                }
                "is-pinned" => {
                    let is_pinned = value.get().unwrap();
                    self.is_pinned.set(is_pinned);
                }
                "is-trashed" => {
                    let is_trashed = value.get().unwrap();
                    self.is_trashed.set(is_trashed);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "tag-list" => self.tag_list.borrow().to_value(),
                "last-modified" => self.last_modified.borrow().to_value(),
                "is-pinned" => self.is_pinned.get().to_value(),
                "is-trashed" => self.is_trashed.get().to_value(),
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

    pub fn set_title(&self, title: String) {
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

    pub fn set_is_trashed(&self, is_trashed: bool) {
        self.set_property("is-trashed", is_trashed).unwrap();
    }

    pub fn is_trashed(&self) -> bool {
        self.property("is-trashed").unwrap().get().unwrap()
    }
}

impl Serialize for Metadata {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::Metadata::from_instance(self);
        imp.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let imp = imp::Metadata::deserialize(deserializer)?;

        let metadata = glib::Object::new::<Self>(&[
            ("title", &*imp.title.borrow()),
            ("tag-list", &*imp.tag_list.borrow()),
            ("last-modified", &*imp.last_modified.borrow()),
            ("is-pinned", &imp.is_pinned.get()),
            ("is-trashed", &imp.is_trashed.get()),
        ])
        .expect("Failed to create Metadata.");

        Ok(metadata)
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}
