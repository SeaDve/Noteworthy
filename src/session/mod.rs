mod content_view;
mod manager;
mod note;
mod sidebar;

use adw::subclass::prelude::*;
use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use std::{cell::RefCell, path::Path};

use self::{
    content_view::ContentView,
    manager::{LocalNotesManager, NotesManagerExt},
    note::{Note, NoteExt},
    sidebar::Sidebar,
};

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/session.ui")]
    pub struct Session {
        #[template_child]
        pub sidebar: TemplateChild<Sidebar>,
        #[template_child]
        pub content_view: TemplateChild<ContentView>,

        pub selected_note: RefCell<Option<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Session {
        const NAME: &'static str = "NwtySession";
        type Type = super::Session;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();

            Sidebar::static_type();
            ContentView::static_type();
        }
    }

    impl ObjectImpl for Session {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let notes_manager = LocalNotesManager::new(Path::new("/home/dave/Notes"));
            let notes_list = notes_manager.retrive_notes().unwrap();

            self.sidebar
                .set_model(Some(&gtk::SingleSelection::new(Some(&notes_list))));

            self.sidebar
                .connect_activate(clone!(@weak obj => move |sidebar, pos| {
                    let selected_note: Note = sidebar
                        .model()
                        .unwrap()
                        .item(pos)
                        .unwrap()
                        .downcast()
                        .unwrap();

                    dbg!(selected_note.title());

                    obj.set_selected_note(Some(selected_note));
                }));
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "selected-note",
                    "Selected Note",
                    "The selected note in this sidebar",
                    Note::static_type(),
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
                "selected-note" => {
                    let selected_note = value.get().unwrap();
                    self.selected_note.replace(selected_note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-note" => self.selected_note.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Session {}
    impl BinImpl for Session {}
}

glib::wrapper! {
    pub struct Session(ObjectSubclass<imp::Session>)
        @extends gtk::Widget, adw::Bin;
}

impl Session {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Session.")
    }

    pub fn set_selected_note(&self, selected_note: Option<Note>) {
        self.set_property("selected-note", selected_note).unwrap();
    }
}
