use gtk::{glib, prelude::*, subclass::prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

use crate::{
    core::DateTime,
    model::{AttachmentList, NoteTagList},
};

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(default)]
    pub struct NoteMetadataInner {
        pub title: String,
        pub tag_list: NoteTagList,
        pub attachment_list: AttachmentList,
        pub last_modified: DateTime,
        pub is_pinned: bool,
        pub is_trashed: bool,
    }

    #[derive(Debug, Default)]
    pub struct NoteMetadata {
        pub inner: RefCell<NoteMetadataInner>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteMetadata {
        const NAME: &'static str = "NwtyNoteMetadata";
        type Type = super::NoteMetadata;
    }

    impl ObjectImpl for NoteMetadata {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "title",
                        "Title",
                        "Title of the note",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "tag-list",
                        "Tag List",
                        "List containing the tags of the note",
                        NoteTagList::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
                        "attachment-list",
                        "Attachment List",
                        "List containing the attachments of the note",
                        AttachmentList::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "last-modified",
                        "Last Modified",
                        "Last modified datetime of the note",
                        DateTime::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-pinned",
                        "Is Pinned",
                        "Whether the note is pinned",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-trashed",
                        "Is Trashed",
                        "Whether the note is in trash",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                    obj.set_title(title);
                }
                "tag-list" => {
                    let tag_list = value.get().unwrap();
                    obj.set_tag_list(tag_list);
                }
                "attachment-list" => {
                    let attachment_list = value.get().unwrap();
                    obj.set_attachment_list(attachment_list);
                }
                "last-modified" => {
                    let last_modified = value.get().unwrap();
                    obj.set_last_modified(last_modified);
                }
                "is-pinned" => {
                    let is_pinned = value.get().unwrap();
                    obj.set_is_pinned(is_pinned);
                }
                "is-trashed" => {
                    let is_trashed = value.get().unwrap();
                    obj.set_is_trashed(is_trashed);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => obj.title().to_value(),
                "tag-list" => obj.tag_list().to_value(),
                "attachment-list" => obj.attachment_list().to_value(),
                "last-modified" => obj.last_modified().to_value(),
                "is-pinned" => obj.is_pinned().to_value(),
                "is-trashed" => obj.is_trashed().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct NoteMetadata(ObjectSubclass<imp::NoteMetadata>);
}

impl NoteMetadata {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NoteMetadata.")
    }

    pub fn set_title(&self, title: &str) {
        if title == self.title() {
            return;
        }

        self.imp().inner.borrow_mut().title = title.to_string();
        self.notify("title");

        self.update_last_modified();
    }

    pub fn title(&self) -> String {
        self.imp().inner.borrow().title.clone()
    }

    pub fn set_tag_list(&self, tag_list: NoteTagList) {
        if tag_list == self.tag_list() {
            return;
        }

        self.imp().inner.borrow_mut().tag_list = tag_list;
        self.notify("tag-list");
    }

    pub fn tag_list(&self) -> NoteTagList {
        self.imp().inner.borrow().tag_list.clone()
    }

    pub fn set_attachment_list(&self, attachment_list: AttachmentList) {
        if attachment_list == self.attachment_list() {
            return;
        }

        self.imp().inner.borrow_mut().attachment_list = attachment_list;
        self.notify("attachment-list");
    }

    pub fn attachment_list(&self) -> AttachmentList {
        self.imp().inner.borrow().attachment_list.clone()
    }

    pub fn set_last_modified(&self, last_modified: &DateTime) {
        if last_modified == &self.last_modified() {
            return;
        }

        self.imp().inner.borrow_mut().last_modified = *last_modified;
        self.notify("last-modified");
    }

    pub fn last_modified(&self) -> DateTime {
        self.imp().inner.borrow().last_modified
    }

    pub fn set_is_pinned(&self, is_pinned: bool) {
        if is_pinned == self.is_pinned() {
            return;
        }

        self.imp().inner.borrow_mut().is_pinned = is_pinned;
        self.notify("is-pinned");
    }

    pub fn is_pinned(&self) -> bool {
        self.imp().inner.borrow().is_pinned
    }

    pub fn set_is_trashed(&self, is_trashed: bool) {
        if is_trashed == self.is_trashed() {
            return;
        }

        self.imp().inner.borrow_mut().is_trashed = is_trashed;
        self.notify("is-trashed");
    }

    pub fn is_trashed(&self) -> bool {
        self.imp().inner.borrow().is_trashed
    }

    pub fn update_last_modified(&self) {
        self.set_last_modified(&DateTime::now());
    }

    pub fn update(&self, other: &Self) {
        self.set_title(&other.title());
        self.set_tag_list(other.tag_list());
        self.set_attachment_list(other.attachment_list());
        self.set_last_modified(&other.last_modified());
        self.set_is_pinned(other.is_pinned());
        self.set_is_trashed(other.is_trashed());
    }
}

impl Serialize for NoteMetadata {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.imp().inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NoteMetadata {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = imp::NoteMetadataInner::deserialize(deserializer)?;

        let metadata = Self::new();
        metadata.imp().inner.replace(inner);

        Ok(metadata)
    }
}

impl Default for NoteMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::{Attachment, Tag};
    use gtk::gio;

    #[test]
    fn title() {
        let metadata = NoteMetadata::new();
        assert_eq!(metadata.title(), "");
        metadata.set_title("A Title");
        assert_eq!(metadata.title(), "A Title");
    }

    #[test]
    fn title_did_not_changed() {
        let metadata = NoteMetadata::new();
        metadata.set_title("Title");
        let old_last_modified = metadata.last_modified();
        metadata.set_title("Title");
        let new_last_modified = metadata.last_modified();
        assert_eq!(old_last_modified, new_last_modified);
    }

    #[test]
    fn title_did_changed() {
        let metadata = NoteMetadata::new();
        metadata.set_title("Title");
        let old_last_modified = metadata.last_modified();
        metadata.set_title("New Title");
        let new_last_modified = metadata.last_modified();
        assert!(old_last_modified < new_last_modified);
    }

    #[test]
    fn tag_list() {
        let metadata = NoteMetadata::new();
        assert!(metadata.tag_list().is_empty());

        let new_tag_list = NoteTagList::new();
        new_tag_list.append(Tag::new("A Tag")).unwrap();

        metadata.set_tag_list(new_tag_list.clone());
        assert!(!metadata.tag_list().is_empty());
        assert_eq!(metadata.tag_list(), new_tag_list);
    }

    #[test]
    fn attachment_list() {
        let metadata = NoteMetadata::new();
        assert!(metadata.attachment_list().is_empty());

        let new_attachment_list = AttachmentList::new();
        new_attachment_list
            .append(Attachment::new(
                &gio::File::for_path("/home/test/t.png"),
                &DateTime::default(),
            ))
            .unwrap();

        metadata.set_attachment_list(new_attachment_list.clone());
        assert!(!metadata.attachment_list().is_empty());
        assert_eq!(metadata.attachment_list(), new_attachment_list);
    }

    #[test]
    fn last_modified() {
        let metadata = NoteMetadata::new();
        assert_eq!(metadata.title(), "");
        metadata.set_title("A Title");
        assert_eq!(metadata.title(), "A Title");
    }

    #[test]
    fn update_last_modified() {
        let metadata = NoteMetadata::new();
        let old_last_modified = metadata.last_modified();
        metadata.update_last_modified();
        assert!(old_last_modified < metadata.last_modified());
    }

    #[test]
    fn is_pinned() {
        let metadata = NoteMetadata::new();
        assert!(!metadata.is_pinned());
        metadata.set_is_pinned(true);
        assert!(metadata.is_pinned());
    }

    #[test]
    fn is_trashed() {
        let metadata = NoteMetadata::new();
        assert!(!metadata.is_trashed());
        metadata.set_is_trashed(true);
        assert!(metadata.is_trashed());
    }

    #[test]
    fn update() {
        let metadata = NoteMetadata::new();
        assert_eq!(metadata.title(), "");
        assert!(metadata.tag_list().is_empty());
        assert!(metadata.attachment_list().is_empty());
        assert!(!metadata.is_pinned());
        assert!(!metadata.is_trashed());

        let other_metadata = NoteMetadata::new();
        other_metadata.set_title("Title");

        let tag_list = NoteTagList::new();
        tag_list.append(Tag::new("A Tag")).unwrap();
        other_metadata.set_tag_list(tag_list);

        let attachment_list = AttachmentList::new();
        attachment_list
            .append(Attachment::new(
                &gio::File::for_path("/home/test/t.png"),
                &DateTime::default(),
            ))
            .unwrap();
        other_metadata.set_attachment_list(attachment_list);

        other_metadata.set_last_modified(&DateTime::now());
        other_metadata.set_is_pinned(true);
        other_metadata.set_is_trashed(true);

        metadata.update(&other_metadata);
        assert_eq!(metadata.title(), other_metadata.title());
        assert!(!metadata.tag_list().is_empty());
        assert!(!metadata.attachment_list().is_empty());
        assert_eq!(metadata.tag_list(), other_metadata.tag_list());
        assert_eq!(metadata.attachment_list(), other_metadata.attachment_list());
        assert_eq!(metadata.last_modified(), other_metadata.last_modified());
        assert_eq!(metadata.is_pinned(), other_metadata.is_pinned());
        assert_eq!(metadata.is_trashed(), other_metadata.is_trashed());
    }
}
