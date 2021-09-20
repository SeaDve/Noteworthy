mod content;
mod note;
mod note_list;
mod note_manager;
mod note_tag_dialog;
mod sidebar;
mod tag_editor;

use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use std::cell::RefCell;

use self::{
    content::Content, note::Note, note_list::NoteList, note_manager::NoteManager,
    note_tag_dialog::NoteTagDialog, sidebar::Sidebar, tag_editor::TagEditor,
};

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

        pub note_manager: OnceCell<NoteManager>,
        pub selected_note: RefCell<Option<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Session {
        const NAME: &'static str = "NwtySession";
        type Type = super::Session;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Sidebar::static_type();
            Content::static_type();
            Self::bind_template(klass);

            klass.install_action("session.edit-tags", None, move |obj, _, _| {
                let imp = imp::Session::from_instance(obj);
                let tag_list = imp.note_manager.get().unwrap().tag_list();

                let tag_editor = TagEditor::new(&tag_list);
                tag_editor.set_modal(true);
                tag_editor.set_transient_for(
                    obj.root()
                        .map(|w| w.downcast::<gtk::Window>().unwrap())
                        .as_ref(),
                );
                tag_editor.present();
            });

            klass.install_action("session.edit-note-tags", None, move |obj, _, _| {
                let imp = imp::Session::from_instance(obj);
                let tag_list = imp.note_manager.get().unwrap().tag_list();
                let other_tag_list = obj.selected_note().unwrap().metadata().tag_list();

                let note_tag_dialog = NoteTagDialog::new(&tag_list, &other_tag_list);
                note_tag_dialog.set_modal(true);
                note_tag_dialog.set_transient_for(
                    obj.root()
                        .map(|w| w.downcast::<gtk::Window>().unwrap())
                        .as_ref(),
                );
                note_tag_dialog.present();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Session {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak obj => async move {
                let note_manager = obj.note_manager();
                note_manager.load_data_file().await.expect("Failed to load data file");
                note_manager.load_notes().await.expect("Failed to load files");

                let imp = imp::Session::from_instance(&obj);
                imp.sidebar.set_note_list(note_manager.note_list());
                imp.sidebar.set_tag_list(note_manager.tag_list());
            }));

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

        // Save previous note before switching to other notes
        self.save_active_note();

        let imp = imp::Session::from_instance(self);

        if selected_note.is_some() {
            imp.leaflet.navigate(adw::NavigationDirection::Forward);
        } else {
            imp.leaflet.navigate(adw::NavigationDirection::Back);
        }

        imp.selected_note.replace(selected_note);
        self.notify("selected-note");
    }

    pub fn note_manager(&self) -> &NoteManager {
        let imp = imp::Session::from_instance(self);
        imp.note_manager.get_or_init(|| {
            let directory = gio::File::for_path("/home/dave/NotesDevel");
            NoteManager::for_directory(&directory)
        })
    }

    // TODO Add autosave
    pub fn save_active_note(&self) {
        if let Some(note) = self.selected_note() {
            let ctx = glib::MainContext::default();
            ctx.spawn_local(clone!(@weak self as obj => async move {
                obj.note_manager().save_note(note).await.unwrap();
            }));
        }
    }

    pub fn save(&self) {
        let note_manager = self.note_manager();
        note_manager
            .save_all_notes()
            .expect("Failed to save notes to file");
        note_manager
            .save_data_file()
            .expect("Failed to save data file");

        log::info!("Session saved");
    }
}
