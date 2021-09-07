mod note_row;

use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::sync::OnceCell;

use std::cell::{Cell, RefCell};

use self::note_row::NoteRow;
use super::{Note, NoteList, Session};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar.ui")]
    pub struct Sidebar {
        #[template_child]
        pub listview: TemplateChild<gtk::ListView>,

        pub compact: Cell<bool>,
        pub selected_note: RefCell<Option<Note>>,
        pub selection: RefCell<Option<gtk::SingleSelection>>,

        pub session: OnceCell<Session>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "NwtySidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            NoteRow::static_type();
            Self::bind_template(klass);

            klass.install_action("sidebar.create-note", None, move |obj, _, _| {
                // FIXME more proper way to create note
                let imp = imp::Sidebar::from_instance(obj);
                imp.session
                    .get()
                    .unwrap()
                    .note_manager()
                    .create_note()
                    .expect("Failed to create note");
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
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
                        NoteList::static_type(),
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

        let filter = gtk::CustomFilter::new(|item| {
            let is_pinned = item.downcast_ref::<Note>().unwrap().metadata().is_pinned();
            true
        });
        let filter_model = gtk::FilterListModel::new(Some(&note_list), Some(&filter));

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let note1_metadata = obj1.downcast_ref::<Note>().unwrap().metadata();
            let note2_metadata = obj2.downcast_ref::<Note>().unwrap().metadata();

            note2_metadata
                .last_modified()
                .cmp(&note1_metadata.last_modified())
                .into()
        });
        let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));

        note_list.connect_note_metadata_changed(
            clone!(@strong filter, @strong sorter => move |_| {
                filter.changed(gtk::FilterChange::Different);
                sorter.changed(gtk::SorterChange::Different);
            }),
        );

        let selection = gtk::SingleSelection::new(Some(&sort_model));
        selection
            .bind_property("selected-item", self, "selected-note")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        imp.selection.replace(Some(selection));

        imp.listview
            .set_model(Some(imp.selection.borrow().as_ref().unwrap()));

        self.set_selected_note(None);
    }

    pub fn set_selected_note(&self, selected_note: Option<Note>) {
        if self.selected_note() == selected_note {
            return;
        }

        let imp = imp::Sidebar::from_instance(self);
        if selected_note.is_none() {
            imp.selection
                .borrow()
                .as_ref()
                .unwrap()
                .set_selected(gtk::INVALID_LIST_POSITION);
        }

        imp.selected_note.replace(selected_note);
        self.notify("selected-note");
    }

    pub fn selected_note(&self) -> Option<Note> {
        let imp = imp::Sidebar::from_instance(self);
        imp.selected_note.borrow().clone()
    }

    // TODO remove this in the future
    pub fn set_session(&self, session: Session) {
        let imp = imp::Sidebar::from_instance(self);
        imp.session.set(session).unwrap();
    }
}
