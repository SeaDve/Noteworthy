mod local_note;
mod notes_list;

pub use local_note::LocalNote;
pub use notes_list::NotesList;

use gtk::{
    glib::{self, prelude::*},
    prelude::*,
    subclass::prelude::*,
};

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Note {
        pub title: RefCell<String>,
        pub content: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Note {
        const NAME: &'static str = "NwtyNote";
        type Type = super::Note;
        type ParentType = glib::Object;
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
                    obj.set_title(title);
                }
                "content" => {
                    let content = value.get().unwrap();
                    obj.set_content(content);
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

pub trait NoteExt: IsA<Note> {
    fn set_title(&self, title: &str);
    fn title(&self) -> String;

    fn set_content(&self, content: &str);
    fn content(&self) -> String;
}

impl<T: IsA<Note>> NoteExt for T {
    // TODO make this call the subclass implementation of NoteImpl
    fn set_title(&self, title: &str) {
        let imp = imp::Note::from_instance(self.upcast_ref());
        imp.replace_title(self.upcast_ref(), title);
    }

    fn title(&self) -> String {
        let imp = imp::Note::from_instance(self.upcast_ref());
        imp.retrieve_title(self.upcast_ref())
    }

    fn set_content(&self, content: &str) {
        let imp = imp::Note::from_instance(self.upcast_ref());
        imp.replace_content(self.upcast_ref(), content);
    }

    fn content(&self) -> String {
        let imp = imp::Note::from_instance(self.upcast_ref());
        imp.retrieve_content(self.upcast_ref())
    }
}

pub trait NoteImpl: ObjectImpl {
    fn replace_title(&self, obj: &Self::Type, title: &str);
    fn retrieve_title(&self, obj: &Self::Type) -> String;

    fn replace_content(&self, obj: &Self::Type, content: &str);
    fn retrieve_content(&self, obj: &Self::Type) -> String;
}

unsafe impl<T: NoteImpl + 'static> IsSubclassable<T> for Note {
    fn class_init(class: &mut glib::Class<Self>) {
        <glib::Object as IsSubclassable<T>>::class_init(class.upcast_ref_mut());
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <glib::Object as IsSubclassable<T>>::instance_init(instance);
    }
}
