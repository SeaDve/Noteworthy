mod content;
mod manager;
mod sidebar;

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*};
use once_cell::sync::OnceCell;

use std::{cell::RefCell, path::Path};

use self::{
    content::Content,
    manager::{Note, NoteManager},
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
        pub leaflet: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub sidebar: TemplateChild<Sidebar>,
        #[template_child]
        pub content: TemplateChild<Content>,

        pub notes_manager: OnceCell<NoteManager>,
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
            Content::static_type();
        }
    }

    impl ObjectImpl for Session {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let note_list = obj.notes_manager().note_list();

            self.sidebar.set_note_list(note_list);
            self.sidebar.set_session(obj.clone());
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "selected-note",
                    "Selected Note",
                    "The selected note",
                    Note::static_type(),
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
                "selected-note" => {
                    let selected_note = value.get().unwrap();
                    obj.set_selected_note(selected_note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-note" => obj.selected_note().to_value(),
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
        let imp = imp::Session::from_instance(self);
        imp.selected_note.borrow().clone()
    }

    pub fn set_selected_note(&self, selected_note: Option<Note>) {
        if self.selected_note() == selected_note {
            return;
        }

        let imp = imp::Session::from_instance(self);

        if selected_note.is_some() {
            imp.leaflet.navigate(adw::NavigationDirection::Forward);
        } else {
            imp.leaflet.navigate(adw::NavigationDirection::Back);
        }

        imp.selected_note.replace(selected_note);
        self.notify("selected-note");
    }

    pub fn notes_manager(&self) -> &NoteManager {
        let imp = imp::Session::from_instance(self);
        imp.notes_manager
            .get_or_init(|| NoteManager::new(Path::new("/home/dave/Notes")))
    }

    pub fn save(&self) -> Result<()> {
        let imp = imp::Session::from_instance(self);
        imp.content.save_active_note();
        self.notes_manager().save_notes_to_file()?;

        log::info!("Session saved");

        Ok(())
    }
}
