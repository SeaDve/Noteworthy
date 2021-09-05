mod note_row;

use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use once_cell::sync::OnceCell;

use std::cell::{Cell, RefCell};

use self::note_row::NoteRow;
use super::{
    manager::{Note, NoteList},
    Session,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar.ui")]
    pub struct Sidebar {
        #[template_child]
        pub listview: TemplateChild<gtk::ListView>,

        pub compact: Cell<bool>,
        pub selected_note: RefCell<Option<Note>>,

        pub session: OnceCell<Session>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "NwtySidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("sidebar.create-note", None, move |obj, _, _| {
                // FIXME more proper way to create note
                let imp = imp::Sidebar::from_instance(obj);
                imp.session
                    .get()
                    .unwrap()
                    .notes_manager()
                    .create_note()
                    .expect("Failed to create note");
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
                        "note-list",
                        "Note List",
                        "Note list represented by self",
                        Note::static_type(),
                        glib::ParamFlags::WRITABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-note",
                        "Selected Note",
                        "The selected note in this sidebar",
                        Note::static_type(),
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
                "compact" => {
                    let compact = value.get().unwrap();
                    self.compact.set(compact);
                }
                "note-list" => {
                    let note_list = value.get().unwrap();
                    obj.set_note_list(note_list);
                }
                "selected-note" => {
                    let selected_note = value.get().unwrap();
                    obj.set_selected_note(selected_note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "selected-note" => obj.selected_note().to_value(),
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

    pub fn set_note_list(&self, note_list: NoteList) {
        let imp = imp::Sidebar::from_instance(self);

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let order1 = obj1.downcast_ref::<Note>().unwrap().modified();
            let order2 = obj2.downcast_ref::<Note>().unwrap().modified();

            order2.cmp(&order1).into()
        });

        let note_expression = gtk::ClosureExpression::new(
            |value| {
                value[0]
                    .get::<gtk::TreeListRow>()
                    .unwrap()
                    .item()
                    .and_then(|o| o.downcast::<Note>().ok())
                    .map_or(String::new(), |o| o.title())
            },
            &[],
        );

        let filter = gtk::StringFilterBuilder::new()
            .match_mode(gtk::StringFilterMatchMode::Substring)
            .expression(&note_expression)
            .ignore_case(true)
            .build();

        let filter_model = gtk::FilterListModel::new(Some(&note_list), Some(&filter));

        imp.listview
            .set_model(Some(&gtk::SingleSelection::new(Some(&filter_model))));
    }

    pub fn set_selected_note(&self, note: Option<Note>) {
        if self.selected_note() == note {
            return;
        }

        let imp = imp::Sidebar::from_instance(self);
        imp.selected_note.replace(note);

        self.notify("selected-note");
    }

    // TODO remove this in the future
    pub fn set_session(&self, session: Session) {
        let imp = imp::Sidebar::from_instance(self);
        imp.session.set(session).unwrap();
    }

    pub fn selected_note(&self) -> Option<Note> {
        let imp = imp::Sidebar::from_instance(self);
        imp.selected_note.borrow().clone()
    }

    pub fn connect_activate(
        &self,
        f: impl Fn(&gtk::ListView, u32) + 'static,
    ) -> glib::SignalHandlerId {
        let imp = imp::Sidebar::from_instance(self);
        imp.listview.connect_activate(f)
    }
}
