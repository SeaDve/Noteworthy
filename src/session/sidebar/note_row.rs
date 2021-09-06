use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use super::super::note::Metadata;
use super::Note;
use crate::date::Date;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar_note_row.ui")]
    pub struct NoteRow {
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub time_label: TemplateChild<gtk::Label>,

        pub note: RefCell<Option<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteRow {
        const NAME: &'static str = "NwtySidebarNoteRow";
        type Type = super::NoteRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NoteRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "note",
                    "Note",
                    "Note represented by self",
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
                "note" => {
                    let note = value.get().unwrap();
                    obj.set_note(note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "note" => obj.note().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for NoteRow {}
    impl BinImpl for NoteRow {}
}

glib::wrapper! {
    pub struct NoteRow(ObjectSubclass<imp::NoteRow>)
        @extends gtk::Widget, adw::Bin;
}

impl NoteRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NoteRow.")
    }

    pub fn note(&self) -> Option<Note> {
        let imp = imp::NoteRow::from_instance(self);
        imp.note.borrow().clone()
    }

    pub fn set_note(&self, note: Option<Note>) {
        if self.note() == note {
            return;
        }

        let imp = imp::NoteRow::from_instance(self);

        if let Some(ref note) = note {
            // Expression describing how to get subtitle label of self from content of note
            let note_expression = gtk::ConstantExpression::new(&note).upcast();
            let content_expression = gtk::PropertyExpression::new(
                Note::static_type(),
                Some(&note_expression),
                "content",
            )
            .upcast();
            let text_expression = gtk::PropertyExpression::new(
                sourceview::Buffer::static_type(),
                Some(&content_expression),
                "text",
            )
            .upcast();
            let subtitle_expression = gtk::ClosureExpression::new(
                |args| {
                    let text: String = args[1].get().unwrap();
                    text.lines().next().unwrap_or_default().to_string()
                },
                &[text_expression],
            )
            .upcast();
            subtitle_expression.bind(&imp.subtitle_label.get(), "label", None);

            // Expression describing how to get time label of self from date of note
            let note_expression = gtk::ConstantExpression::new(&note).upcast();
            let metadata_expression = gtk::PropertyExpression::new(
                Note::static_type(),
                Some(&note_expression),
                "metadata",
            )
            .upcast();
            let modified_expression = gtk::PropertyExpression::new(
                Metadata::static_type(),
                Some(&metadata_expression),
                "modified",
            )
            .upcast();
            let time_expression = gtk::ClosureExpression::new(
                |args| {
                    let modified: Date = args[1].get().unwrap();
                    modified.fuzzy_display()
                },
                &[modified_expression],
            )
            .upcast();
            time_expression.bind(&imp.time_label.get(), "label", None);
        }

        imp.note.replace(note);
        self.notify("note");
    }
}
