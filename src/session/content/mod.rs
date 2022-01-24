mod attachment_view;
mod view;

use gtk::{glib, prelude::*, subclass::prelude::*};

use std::cell::{Cell, RefCell};

use self::{attachment_view::AttachmentView, view::View};
use crate::model::Note;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content.ui")]
    pub struct Content {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub view_flap: TemplateChild<adw::Flap>,
        #[template_child]
        pub attachment_view: TemplateChild<AttachmentView>,
        #[template_child]
        pub no_selected_view: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub edit_tags_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub is_pinned_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub is_trashed_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub view_flap_button: TemplateChild<gtk::ToggleButton>,

        pub compact: Cell<bool>,
        pub note: RefCell<Option<Note>>,

        pub bindings: RefCell<Vec<glib::Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Content {
        const NAME: &'static str = "NwtyContent";
        type Type = super::Content;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            View::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Content {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::new(
                        "compact",
                        "Compact",
                        "Whether it is compact view mode",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "note",
                        "Note",
                        "Current note in the view",
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
                "note" => {
                    let note = value.get().unwrap();
                    obj.set_note(note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "note" => obj.note().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.update_buttons_visibility();
            obj.update_stack();
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Content {}
}

glib::wrapper! {
    pub struct Content(ObjectSubclass<imp::Content>)
        @extends gtk::Widget;
}

impl Content {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Content.")
    }

    pub fn note(&self) -> Option<Note> {
        self.imp().note.borrow().clone()
    }

    pub fn set_note(&self, note: Option<Note>) {
        if self.note() == note {
            return;
        }

        let imp = self.imp();

        for binding in imp.bindings.borrow_mut().drain(..) {
            binding.unbind();
        }

        if let Some(ref note) = note {
            let mut bindings = imp.bindings.borrow_mut();
            let note_metadata = note.metadata();

            let is_pinned = note_metadata
                .bind_property("is-pinned", &imp.is_pinned_button.get(), "active")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();
            bindings.push(is_pinned);

            let is_trashed = note_metadata
                .bind_property("is-trashed", &imp.is_trashed_button.get(), "active")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();
            bindings.push(is_trashed);
        }

        imp.note.replace(note);
        self.notify("note");

        self.update_buttons_visibility();
        self.update_stack();
    }

    fn update_stack(&self) {
        let imp = self.imp();

        if self.note().is_some() {
            imp.stack.set_visible_child(&imp.view_flap.get());
        } else {
            imp.stack.set_visible_child(&imp.no_selected_view.get());
        }
    }

    fn update_buttons_visibility(&self) {
        let imp = self.imp();
        let has_note = self.note().is_some();

        imp.is_pinned_button.set_visible(has_note);
        imp.is_trashed_button.set_visible(has_note);
        imp.edit_tags_button.set_visible(has_note);
        imp.view_flap_button.set_visible(has_note);
    }
}
