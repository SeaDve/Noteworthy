mod content;
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

use std::cell::{Cell, RefCell};

use self::{
    content::Content,
    note_tag_dialog::{NoteTagDialog, NoteTagLists},
    sidebar::Sidebar,
    tag_editor::TagEditor,
};
use crate::{core::NoteManager, model::Note};

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
        pub is_syncing: Cell<bool>,
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

            klass.install_action("session.sync", None, move |obj, _, _| {
                let note_manager = obj.note_manager();
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak note_manager => async move {
                    note_manager.sync().await.expect("Failed to sync");
                }));
            });

            klass.install_action("session.create-note", None, move |obj, _, _| {
                let note_manager = obj.note_manager();
                note_manager.create_note().expect("Failed to create note");
            });

            klass.install_action("session.edit-tags", None, move |obj, _, _| {
                let tag_list = obj.note_manager().tag_list();
                let note_list = obj.note_manager().note_list();

                let tag_editor = TagEditor::new(&tag_list, &note_list);
                tag_editor.set_modal(true);
                tag_editor.set_transient_for(
                    obj.root()
                        .map(|w| w.downcast::<gtk::Window>().unwrap())
                        .as_ref(),
                );
                tag_editor.present();
            });

            klass.install_action("session.edit-selected-note-tags", None, move |obj, _, _| {
                let imp = imp::Session::from_instance(obj);
                let tag_list = imp.note_manager.get().unwrap().tag_list();
                let selected_note_tag_list =
                    imp.sidebar.selected_note().unwrap().metadata().tag_list();

                let note_tag_dialog = NoteTagDialog::new(
                    &tag_list,
                    &NoteTagLists::from(vec![selected_note_tag_list]),
                );
                note_tag_dialog.set_modal(true);
                note_tag_dialog.set_transient_for(
                    obj.root()
                        .map(|w| w.downcast::<gtk::Window>().unwrap())
                        .as_ref(),
                );
                note_tag_dialog.present();
            });

            klass.install_action(
                "session.edit-multi-selected-note-tags",
                None,
                move |obj, _, _| {
                    let imp = imp::Session::from_instance(obj);
                    let tag_list = imp.note_manager.get().unwrap().tag_list();
                    let other_tag_lists = imp
                        .sidebar
                        .selected_notes()
                        .iter()
                        .map(|note| note.metadata().tag_list())
                        .collect::<Vec<_>>();

                    let note_tag_dialog =
                        NoteTagDialog::new(&tag_list, &NoteTagLists::from(other_tag_lists));
                    note_tag_dialog.set_modal(true);
                    note_tag_dialog.set_transient_for(
                        obj.root()
                            .map(|w| w.downcast::<gtk::Window>().unwrap())
                            .as_ref(),
                    );
                    note_tag_dialog.present();
                },
            );
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
                note_manager.load().await.expect("Failed to load notes and data file");

                let imp = imp::Session::from_instance(&obj);
                imp.sidebar.set_note_list(note_manager.note_list());

                // FIXME do binding here because we update the tag list again
                // or maybe find other workarounds
                imp.sidebar.set_tag_list(note_manager.tag_list());

                note_manager.sync().await.expect("Failed to sync notes and data file");
            }));

            obj.note_manager()
                .bind_property("is-syncing", obj, "is-syncing")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "note-manager",
                        "Note Manager",
                        "Manages the notes",
                        NoteManager::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-note",
                        "Selected Note",
                        "The selected note",
                        Note::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-syncing",
                        "Is Syncing",
                        "Whether the session is currently syncing",
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
                "note-manager" => {
                    let note_manager = value.get().unwrap();
                    obj.set_note_manager(note_manager);
                }
                "selected-note" => {
                    let selected_note = value.get().unwrap();
                    obj.set_selected_note(selected_note);
                }
                "is-syncing" => {
                    let is_syncing = value.get().unwrap();
                    self.is_syncing.set(is_syncing);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "note-manager" => obj.note_manager().to_value(),
                "selected-note" => obj.selected_note().to_value(),
                "is-syncing" => self.is_syncing.get().to_value(),
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
    pub async fn new(directory: &gio::File) -> Self {
        let note_manager = NoteManager::for_directory(directory).await;
        glib::Object::new(&[("note-manager", &note_manager)]).expect("Failed to create Session.")
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
        imp.note_manager.get().unwrap()
    }

    pub fn set_note_manager(&self, note_manager: NoteManager) {
        let imp = imp::Session::from_instance(self);
        imp.note_manager.set(note_manager).unwrap();
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
            .save_all_notes_sync()
            .expect("Failed to save notes to file");
        note_manager
            .save_data_file_sync()
            .expect("Failed to save data file");

        log::info!("Session saved");
    }
}
