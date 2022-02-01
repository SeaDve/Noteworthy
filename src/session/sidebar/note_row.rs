use gtk::{
    glib::{self, clone, closure},
    prelude::*,
    subclass::prelude::*,
};

use std::cell::{Cell, RefCell};

use super::{Note, Selection, SelectionMode, Sidebar};
use crate::{core::DateTime, model::NoteMetadata};

const MAX_SUBTITLE_LEN: usize = 100;
const MAX_SUBTITLE_LINE: u32 = 3;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

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
        pub is_selected: Cell<bool>,
        pub position: Cell<u32>,
        pub note: RefCell<Option<Note>>,

        pub buffer_changed_handler_id: RefCell<Option<glib::SignalHandlerId>>,
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
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecEnum::new(
                        "selection-mode",
                        "Selection Mode",
                        "Current selection mode",
                        SelectionMode::static_type(),
                        SelectionMode::default() as i32,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-selected",
                        "Is Checked",
                        "Whether this row is selected",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecUInt::new(
                        "position",
                        "Position",
                        "Position of the item",
                        0,
                        u32::MAX,
                        gtk::INVALID_LIST_POSITION,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
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
                "is-selected" => {
                    let is_selected = value.get().unwrap();
                    obj.set_is_selected(is_selected);
                }
                "position" => {
                    let position = value.get().unwrap();
                    obj.set_position(position);
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
                "is-selected" => obj.is_selected().to_value(),
                "position" => obj.position().to_value(),
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
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NoteRow.")
    }

    pub fn is_selected(&self) -> bool {
        self.imp().is_selected.get()
    }

    pub fn set_is_selected(&self, is_selected: bool) {
        let imp = self.imp();
        imp.check_button.set_active(is_selected);
        imp.is_selected.set(is_selected);
        self.notify("is-selected");
    }

    pub fn position(&self) -> u32 {
        self.imp().position.get()
    }

    pub fn set_position(&self, position: u32) {
        self.imp().position.set(position);
        self.notify("position");
    }

    pub fn selection_mode(&self) -> SelectionMode {
        self.imp().selection_mode.get()
    }

    pub fn set_selection_mode(&self, selection_mode: SelectionMode) {
        let imp = self.imp();

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

    pub fn note(&self) -> Option<Note> {
        self.imp().note.borrow().clone()
    }

    pub fn set_note(&self, note: Option<Note>) {
        let imp = self.imp();

        if let Some(ref note) = note {
            imp.buffer_changed_handler_id
                .replace(Some(note.buffer().connect_changed(
                    clone!(@weak self as obj => move |_| {
                        obj.update_subtitle_label();
                    }),
                )));
        }

        imp.note.replace(note);
        self.notify("note");

        self.update_subtitle_label();
    }

    // TODO remove this, maybe just emit a signal from NoteRow and let sidebar handle changing
    // the selection model
    fn parent_model(&self) -> Selection {
        self.ancestor(Sidebar::static_type())
            .expect("Cannot find `Sidebar` as `NoteRow` ancestor")
            .downcast_ref::<Sidebar>()
            .unwrap()
            .selection_model()
    }

    fn update_subtitle_label(&self) {
        let note = match self.note() {
            Some(note) => note,
            None => return,
        };

        let mut iter = note.buffer().start_iter();
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

        // Panicking when subtitle contains `\u{0}`
        let trimmed_subtitle = subtitle.trim_matches(char::from(0));

        self.imp().subtitle_label.set_label(trimmed_subtitle);
    }

    fn setup_expressions(&self) {
        Self::this_expression("note")
            .chain_property::<Note>("metadata")
            .chain_property::<NoteMetadata>("last-modified")
            .chain_closure::<String>(closure!(|_: Self, last_modified: DateTime| {
                last_modified.fuzzy_display()
            }))
            .bind(&self.imp().time_label.get(), "label", Some(self));
    }

    fn setup_signals(&self) {
        self.imp().check_button.connect_active_notify(
            clone!(@weak self as obj => move |check_button| {
                if obj.selection_mode() != SelectionMode::Multi {
                    return;
                }

                let model = obj.parent_model();

                if check_button.is_active() {
                    model.select_item(obj.position(), false);
                } else {
                    model.unselect_item(obj.position());
                }
            }),
        );

        let gesture_click = gtk::GestureClick::new();
        gesture_click.set_button(3);
        gesture_click.connect_pressed(clone!(@weak self as obj => move |_, _, _, _| {
            let model = obj.parent_model();
            model.set_selection_mode(SelectionMode::Multi);
            model.select_item(obj.position(), true);
        }));
        self.add_controller(&gesture_click);
    }
}
