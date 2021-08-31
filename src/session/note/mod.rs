mod local_note;
mod notes_list;

pub use local_note::LocalNote;
pub use notes_list::NotesList;

use gtk::{
    glib::{self, prelude::*},
    subclass::prelude::*,
};

use std::cell::RefCell;

use crate::Result;

mod imp {
    use super::*;

    pub type NoteInstance = super::Note;

    #[repr(C)]
    pub struct NoteClass {
        pub parent_class: glib::gobject_ffi::GObjectClass,

        pub replace_title: Option<unsafe fn(&NoteInstance, title: &str) -> Result<()>>,
        pub retrieve_title: Option<unsafe fn(&NoteInstance) -> Result<String>>,

        pub replace_content: Option<unsafe fn(&NoteInstance, content: &str) -> Result<()>>,
        pub retrieve_content: Option<unsafe fn(&NoteInstance) -> Result<String>>,
    }

    unsafe impl ClassStruct for NoteClass {
        type Type = Note;
    }

    fn replace_title_default_trampoline(this: &NoteInstance, title: &str) -> Result<()> {
        Note::from_instance(this).replace_title(this, title)
    }

    fn retrieve_title_default_trampoline(this: &NoteInstance) -> Result<String> {
        Note::from_instance(this).retrieve_title(this)
    }

    fn replace_content_default_trampoline(this: &NoteInstance, content: &str) -> Result<()> {
        Note::from_instance(this).replace_content(this, content)
    }

    fn retrieve_content_default_trampoline(this: &NoteInstance) -> Result<String> {
        Note::from_instance(this).retrieve_content(this)
    }

    pub(super) unsafe fn note_replace_title(this: &NoteInstance, title: &str) -> Result<()> {
        let klass = &*(this.class() as *const _ as *const NoteClass);
        (klass.replace_title.unwrap())(this, title)
    }

    pub(super) unsafe fn note_retrieve_title(this: &NoteInstance) -> Result<String> {
        let klass = &*(this.class() as *const _ as *const NoteClass);
        (klass.retrieve_title.unwrap())(this)
    }

    pub(super) unsafe fn note_replace_content(this: &NoteInstance, content: &str) -> Result<()> {
        let klass = &*(this.class() as *const _ as *const NoteClass);
        (klass.replace_content.unwrap())(this, content)
    }

    pub(super) unsafe fn note_retrieve_content(this: &NoteInstance) -> Result<String> {
        let klass = &*(this.class() as *const _ as *const NoteClass);
        (klass.retrieve_content.unwrap())(this)
    }

    #[derive(Debug, Default)]
    pub struct Note {
        pub title: RefCell<String>,
        pub content: RefCell<String>,
    }

    /// Default implementations
    impl Note {
        fn replace_title(&self, _obj: &super::Note, _title: &str) -> Result<()> {
            unimplemented!("Replace title is not implemented")
        }

        fn retrieve_title(&self, _obj: &super::Note) -> Result<String> {
            unimplemented!("Retrieve title is not implemented")
        }

        fn replace_content(&self, _obj: &super::Note, _title: &str) -> Result<()> {
            unimplemented!("Replace content is not implemented")
        }

        fn retrieve_content(&self, _obj: &super::Note) -> Result<String> {
            unimplemented!("Retrieve content is not implemented")
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Note {
        const NAME: &'static str = "NwtyNote";
        type Type = super::Note;
        type ParentType = glib::Object;
        type Class = NoteClass;

        fn class_init(klass: &mut Self::Class) {
            klass.replace_title = Some(replace_title_default_trampoline);
            klass.retrieve_title = Some(retrieve_title_default_trampoline);
            klass.replace_content = Some(replace_content_default_trampoline);
            klass.retrieve_content = Some(retrieve_content_default_trampoline);
        }
    }

    impl ObjectImpl for Note {
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
                        "Title of the notes",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "content",
                        "Content",
                        "Content of the note",
                        None,
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
                    obj.set_title(title).expect("Failed to set note title");
                }
                "content" => {
                    let content = value.get().unwrap();
                    obj.set_content(content)
                        .expect("Failed to set note content");
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => obj.title().to_value(),
                "content" => obj.content().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Note(ObjectSubclass<imp::Note>);
}

impl Note {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).expect("Failed to create Note.")
    }
}

pub trait NoteExt {
    fn set_title(&self, title: &str) -> Result<()>;
    fn title(&self) -> String;

    fn set_content(&self, content: &str) -> Result<()>;
    fn content(&self) -> String;
}

impl<O: IsA<Note>> NoteExt for O {
    fn set_title(&self, title: &str) -> Result<()> {
        unsafe { imp::note_replace_title(self.upcast_ref::<Note>(), title) }
    }

    fn title(&self) -> String {
        unsafe { imp::note_retrieve_title(self.upcast_ref::<Note>()) }.unwrap_or_default()
    }

    fn set_content(&self, content: &str) -> Result<()> {
        unsafe { imp::note_replace_content(self.upcast_ref::<Note>(), content) }
    }

    fn content(&self) -> String {
        unsafe { imp::note_retrieve_content(self.upcast_ref::<Note>()) }.unwrap_or_default()
    }
}

pub trait NoteImpl: ObjectImpl + 'static {
    fn replace_title(&self, obj: &Note, title: &str) -> Result<()> {
        self.parent_replace_title(obj, title)
    }

    fn retrieve_title(&self, obj: &Note) -> Result<String> {
        self.parent_retrieve_title(obj)
    }

    fn replace_content(&self, obj: &Note, content: &str) -> Result<()> {
        self.parent_replace_content(obj, content)
    }

    fn retrieve_content(&self, obj: &Note) -> Result<String> {
        self.parent_retrieve_content(obj)
    }
}

pub trait NoteImplExt: ObjectSubclass {
    fn parent_replace_title(&self, obj: &Note, title: &str) -> Result<()>;
    fn parent_retrieve_title(&self, obj: &Note) -> Result<String>;
    fn parent_replace_content(&self, obj: &Note, content: &str) -> Result<()>;
    fn parent_retrieve_content(&self, obj: &Note) -> Result<String>;
}

impl<T: NoteImpl> NoteImplExt for T {
    fn parent_replace_title(&self, obj: &Note, title: &str) -> Result<()> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut imp::NoteClass;
            if let Some(ref f) = (*parent_class).replace_title {
                f(obj, title)
            } else {
                unimplemented!()
            }
        }
    }

    fn parent_retrieve_title(&self, obj: &Note) -> Result<String> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut imp::NoteClass;
            if let Some(ref f) = (*parent_class).retrieve_title {
                f(obj)
            } else {
                unimplemented!()
            }
        }
    }

    fn parent_replace_content(&self, obj: &Note, content: &str) -> Result<()> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut imp::NoteClass;
            if let Some(ref f) = (*parent_class).replace_content {
                f(obj, content)
            } else {
                unimplemented!()
            }
        }
    }

    fn parent_retrieve_content(&self, obj: &Note) -> Result<String> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut imp::NoteClass;
            if let Some(ref f) = (*parent_class).retrieve_content {
                f(obj)
            } else {
                unimplemented!()
            }
        }
    }
}

unsafe impl<T: NoteImpl> IsSubclassable<T> for Note {
    fn class_init(class: &mut glib::Class<Self>) {
        <glib::Object as IsSubclassable<T>>::class_init(class.upcast_ref_mut());

        let klass = class.as_mut();
        klass.replace_title = Some(replace_title_trampoline::<T>);
        klass.retrieve_title = Some(retrieve_title_trampoline::<T>);
        klass.replace_content = Some(replace_content_trampoline::<T>);
        klass.retrieve_content = Some(retrieve_content_trampoline::<T>);
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <glib::Object as IsSubclassable<T>>::instance_init(instance);
    }
}

unsafe fn replace_title_trampoline<T>(this: &Note, title: &str) -> Result<()>
where
    T: ObjectSubclass + NoteImpl,
{
    let instance = &*(this as *const _ as *const T::Instance);
    let imp = instance.impl_();
    imp.replace_title(this, title)
}

unsafe fn retrieve_title_trampoline<T>(this: &Note) -> Result<String>
where
    T: ObjectSubclass + NoteImpl,
{
    let instance = &*(this as *const _ as *const T::Instance);
    let imp = instance.impl_();
    imp.retrieve_title(this)
}

unsafe fn replace_content_trampoline<T>(this: &Note, content: &str) -> Result<()>
where
    T: ObjectSubclass + NoteImpl,
{
    let instance = &*(this as *const _ as *const T::Instance);
    let imp = instance.impl_();
    imp.replace_content(this, content)
}

unsafe fn retrieve_content_trampoline<T>(this: &Note) -> Result<String>
where
    T: ObjectSubclass + NoteImpl,
{
    let instance = &*(this as *const _ as *const T::Instance);
    let imp = instance.impl_();
    imp.retrieve_content(this)
}
