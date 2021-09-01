mod note_row;

use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use once_cell::sync::OnceCell;

use std::cell::{Cell, RefCell};

use super::{Note, NoteExt, Session};
use note_row::NoteRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar.ui")]
    pub struct Sidebar {
        #[template_child]
        pub listview: TemplateChild<gtk::ListView>,

        pub session: OnceCell<Session>,
        pub compact: Cell<bool>,
        pub selected_note: RefCell<Option<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "NwtySidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("sidebar.create-note", None, move |obj, _, _| {
                let imp = obj.private();

                let new_note = imp.session.get().unwrap().create_note("THis is the title");
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();

            NoteRow::static_type();
        }
    }

    impl ObjectImpl for Sidebar {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boolean(
                        "compact",
                        "Compact",
                        "Whether it is compact view mode",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-note",
                        "Selected Note",
                        "The selected note in this sidebar",
                        Note::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "session",
                        "Session",
                        "Current session",
                        Note::static_type(),
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
                "compact" => {
                    let compact = value.get().unwrap();
                    self.compact.set(compact);
                }
                "selected-note" => {
                    let selected_note = value.get().unwrap();
                    self.selected_note.replace(selected_note);
                }
                "session" => {
                    let session = value.get().unwrap();
                    obj.set_session(session);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "selected-note" => obj.selected_note().to_value(),
                "session" => self.session.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Sidebar {}
    impl BoxImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget, gtk::Box;
}

impl Sidebar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Sidebar.")
    }

    pub fn set_model(&self, model: Option<&impl IsA<gtk::SelectionModel>>) {
        let imp = self.private();
        imp.listview.set_model(model);
    }

    pub fn model(&self) -> Option<impl IsA<gtk::SelectionModel>> {
        let imp = self.private();
        imp.listview.model()
    }

    pub fn set_session(&self, session: Session) {
        let imp = self.private();
        imp.session.set(session).unwrap();
    }

    pub fn selected_note(&self) -> Note {
        let imp = self.private();
        imp.selected_note.borrow().clone().unwrap()
    }

    pub fn connect_activate(
        &self,
        f: impl Fn(&gtk::ListView, u32) + 'static,
    ) -> glib::SignalHandlerId {
        let imp = self.private();
        imp.listview.connect_activate(f)
    }

    fn private(&self) -> &imp::Sidebar {
        imp::Sidebar::from_instance(self)
    }
}
