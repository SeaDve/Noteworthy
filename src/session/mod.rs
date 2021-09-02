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
use once_cell::sync::OnceCell;

use std::cell::RefCell;

use self::{
    content_view::ContentView,
    manager::LocalNotesManager,
    note::{Note, NoteExt},
    sidebar::Sidebar,
};
use crate::Result;

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

        pub notes_manager: OnceCell<LocalNotesManager>,
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

            self.sidebar.set_session(obj.clone());
            self.content_view.set_session(obj.clone());

            let note_list = obj.notes_manager().note_list();

            self.sidebar.set_note_list(Some(note_list));

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
                    "The selected note",
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

    pub fn selected_note(&self) -> Option<Note> {
        self.property("selected-note").unwrap().get().unwrap()
    }

    pub fn set_selected_note(&self, selected_note: Option<Note>) {
        self.set_property("selected-note", selected_note).unwrap();
    }

    pub fn notes_manager(&self) -> &LocalNotesManager {
        let imp = imp::Session::from_instance(self);
        imp.notes_manager
            .get_or_init(|| LocalNotesManager::new("/home/dave/Notes"))
    }

    pub fn save(&self) -> Result<()> {
        let imp = imp::Session::from_instance(self);
        imp.content_view.save_active_note()?;
        Ok(())
    }

    pub fn create_note(&self, title: &str) -> Result<Note> {
        self.notes_manager().create_note(title)
    }
}
