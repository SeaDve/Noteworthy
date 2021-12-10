mod tag_bar;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use gtk_source::prelude::*;

use std::cell::RefCell;

use self::tag_bar::TagBar;
use crate::{
    core::DateTime,
    model::Note,
    utils::{ChainExpr, PropExpr},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-view.ui")]
    pub struct View {
        #[template_child]
        pub title_label: TemplateChild<gtk_source::View>,
        #[template_child]
        pub last_modified_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tag_bar: TemplateChild<TagBar>,
        #[template_child]
        pub source_view: TemplateChild<gtk_source::View>,

        pub bindings: RefCell<Vec<glib::Binding>>,

        pub note: RefCell<Option<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for View {
        const NAME: &'static str = "NwtyContentView";
        type Type = super::View;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for View {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "note",
                    "Note",
                    "Current note in the view",
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

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // For some reason Buffer:style-scheme default is set to something making it
            // not follow libadwaita's StyleManager:is-dark
            let title_label_buffer = self
                .title_label
                .buffer()
                .downcast::<gtk_source::Buffer>()
                .unwrap();
            title_label_buffer.set_style_scheme(None);

            obj.setup_expressions();
        }
    }

    impl WidgetImpl for View {}
    impl BinImpl for View {}
}

glib::wrapper! {
    pub struct View(ObjectSubclass<imp::View>)
        @extends gtk::Widget, adw::Bin;
}

impl View {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create View.")
    }

    pub fn note(&self) -> Option<Note> {
        let imp = imp::View::from_instance(self);
        imp.note.borrow().clone()
    }

    pub fn set_note(&self, note: Option<Note>) {
        let imp = imp::View::from_instance(self);

        for binding in imp.bindings.borrow_mut().drain(..) {
            binding.unbind();
        }

        if let Some(ref note) = note {
            imp.source_view.grab_focus();

            let mut bindings = imp.bindings.borrow_mut();

            let title_binding = note
                .metadata()
                .bind_property("title", &imp.title_label.get().buffer(), "text")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build()
                .unwrap();
            bindings.push(title_binding);

            let buffer_binding = note
                .bind_property("buffer", &imp.source_view.get(), "buffer")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build()
                .unwrap();
            bindings.push(buffer_binding);
        }

        imp.note.replace(note);
        self.notify("note");
    }

    fn setup_expressions(&self) {
        let imp = imp::View::from_instance(self);

        self.property_expression("note")
            .property_expression("metadata")
            .property_expression("last-modified")
            .closure_expression(|args| {
                let last_modified: DateTime = args[1].get().unwrap();
                gettext!("Last edited {}", last_modified.fuzzy_display())
            })
            .bind(
                &imp.last_modified_label.get(),
                "label",
                None::<&gtk::Widget>,
            );
    }
}
