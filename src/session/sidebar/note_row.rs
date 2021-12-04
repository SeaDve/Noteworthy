use adw::subclass::prelude::*;
use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::unsync::OnceCell;

use std::cell::{Cell, RefCell};

use super::{Note, SelectionMode, Sidebar};
use crate::{
    model::DateTime,
    utils::{ChainExpr, PropExpr},
};

const MAX_SUBTITLE_LEN: usize = 100;
const MAX_SUBTITLE_LINE: u32 = 3;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar-note-row.ui")]
    pub struct NoteRow {
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub time_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub check_button_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub check_button: TemplateChild<gtk::CheckButton>,

        pub selection_mode: Cell<SelectionMode>,
        pub is_checked: Cell<bool>,
        pub position: Cell<u32>,
        pub note: RefCell<Option<Note>>,
        pub sidebar: OnceCell<Sidebar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteRow {
        const NAME: &'static str = "NwtySidebarNoteRow";
        type Type = super::NoteRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NoteRow {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_enum(
                        "selection-mode",
                        "Selection Mode",
                        "Current selection mode",
                        SelectionMode::static_type(),
                        SelectionMode::default() as i32,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-checked",
                        "Is Checked",
                        "Whether this row is checked",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_uint(
                        "position",
                        "Position",
                        "Position of the item",
                        0,
                        u32::MAX,
                        gtk::INVALID_LIST_POSITION,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "sidebar",
                        "Sidebar",
                        "The sidebar holding this row",
                        Sidebar::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "note",
                        "Note",
                        "Note represented by self",
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
                "selection-mode" => {
                    let selection_mode = value.get().unwrap();
                    obj.set_selection_mode(selection_mode);
                }
                "is-checked" => {
                    let is_checked = value.get().unwrap();
                    obj.set_is_checked(is_checked);
                }
                "position" => {
                    let position = value.get().unwrap();
                    obj.set_position(position);
                }
                "sidebar" => {
                    let sidebar = value.get().unwrap();
                    obj.set_sidebar(sidebar);
                }
                "note" => {
                    let note = value.get().unwrap();
                    obj.set_note(note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selection-mode" => obj.selection_mode().to_value(),
                "is-checked" => obj.is_checked().to_value(),
                "position" => obj.position().to_value(),
                "sidebar" => obj.sidebar().to_value(),
                "note" => obj.note().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_expressions();
            obj.setup_signals();
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for NoteRow {}
}

glib::wrapper! {
    pub struct NoteRow(ObjectSubclass<imp::NoteRow>)
        @extends gtk::Widget;
}

impl NoteRow {
    pub fn new(sidebar: &Sidebar) -> Self {
        glib::Object::new(&[("sidebar", sidebar)]).expect("Failed to create NoteRow.")
    }

    pub fn is_checked(&self) -> bool {
        let imp = imp::NoteRow::from_instance(self);
        imp.is_checked.get()
    }

    pub fn set_is_checked(&self, is_checked: bool) {
        let imp = imp::NoteRow::from_instance(self);
        imp.check_button.set_active(is_checked);
        imp.is_checked.set(is_checked);
        self.notify("is-checked");
    }

    pub fn position(&self) -> u32 {
        let imp = imp::NoteRow::from_instance(self);
        imp.position.get()
    }

    pub fn set_position(&self, position: u32) {
        let imp = imp::NoteRow::from_instance(self);
        imp.position.set(position);
        self.notify("position");
    }

    pub fn selection_mode(&self) -> SelectionMode {
        let imp = imp::NoteRow::from_instance(self);
        imp.selection_mode.get()
    }

    pub fn set_selection_mode(&self, selection_mode: SelectionMode) {
        let imp = imp::NoteRow::from_instance(self);

        match selection_mode {
            SelectionMode::Single => {
                imp.check_button_revealer.set_reveal_child(false);
            }
            SelectionMode::Multi => {
                imp.check_button_revealer.set_reveal_child(true);
            }
        }

        imp.selection_mode.replace(selection_mode);
        self.notify("selection-mode");
    }

    pub fn sidebar(&self) -> Sidebar {
        let imp = imp::NoteRow::from_instance(self);
        imp.sidebar.get().unwrap().clone()
    }

    fn set_sidebar(&self, sidebar: Sidebar) {
        let imp = imp::NoteRow::from_instance(self);
        imp.sidebar.set(sidebar).unwrap();
    }

    pub fn note(&self) -> Option<Note> {
        let imp = imp::NoteRow::from_instance(self);
        imp.note.borrow().clone()
    }

    pub fn set_note(&self, note: Option<Note>) {
        let imp = imp::NoteRow::from_instance(self);
        imp.note.replace(note);
        self.notify("note");
    }

    fn setup_expressions(&self) {
        let imp = imp::NoteRow::from_instance(self);

        // Expression describing how to get subtitle label of self from buffer of note
        let note_expression = self.property_expression("note");

        note_expression
            .property_expression("buffer")
            .closure_expression(|args| {
                let buffer: gtk_source::Buffer = args[1].get().unwrap();
                let mut iter = buffer.start_iter();
                let mut subtitle = String::from(iter.char());

                let mut line_count = 0;
                let mut last_non_empty_char_index = 0;

                while iter.forward_char() {
                    if subtitle.len() >= MAX_SUBTITLE_LEN || line_count >= MAX_SUBTITLE_LINE {
                        break;
                    }

                    let character = iter.char();

                    if character == '\n' {
                        line_count += 1;
                    }

                    subtitle.push(character);

                    if !character.is_whitespace() {
                        last_non_empty_char_index = subtitle.len() - 1;
                    }
                }

                subtitle.truncate(last_non_empty_char_index + 1);
                subtitle
            })
            .bind(&imp.subtitle_label.get(), "label", None::<&gtk::Widget>);

        // Expression describing how to get time label of self from last_modifed of note
        note_expression
            .property_expression("metadata")
            .property_expression("last-modified")
            .closure_expression(|args| {
                let last_modified: DateTime = args[1].get().unwrap();
                last_modified.fuzzy_display()
            })
            .bind(&imp.time_label.get(), "label", None::<&gtk::Widget>);
    }

    fn setup_signals(&self) {
        let imp = imp::NoteRow::from_instance(self);
        imp.check_button
            .connect_active_notify(clone!(@weak self as obj => move |check_button| {
                if obj.selection_mode() != SelectionMode::Multi {
                    return;
                }

                let model = obj.sidebar().selection_model();

                if check_button.is_active() {
                    model.select_item(obj.position(), false);
                } else {
                    model.unselect_item(obj.position());
                }
            }));

        let gesture_click = gtk::GestureClick::new();
        gesture_click.set_button(3);
        gesture_click.connect_pressed(clone!(@weak self as obj => move |_, _, _, _| {
            let model = obj.sidebar().selection_model();
            model.set_selection_mode(SelectionMode::Multi);
            model.select_item(obj.position(), true);
        }));
        self.add_controller(&gesture_click);
    }
}
