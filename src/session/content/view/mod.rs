mod tag_list_view;

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use self::tag_list_view::TagListView;
use crate::model::{note::Metadata, Date, Note};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-view.ui")]
    pub struct View {
        #[template_child]
        pub title_label: TemplateChild<sourceview::View>,
        #[template_child]
        pub last_modified_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tag_list_view: TemplateChild<TagListView>,
        #[template_child]
        pub source_view: TemplateChild<sourceview::View>,

        pub bindings: RefCell<Vec<glib::Binding>>,

        pub note: RefCell<Option<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for View {
        const NAME: &'static str = "NwtyContentView";
        type Type = super::View;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            TagListView::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for View {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let self_expression = gtk::ConstantExpression::new(&obj);
            let note_expression = gtk::PropertyExpression::new(
                Self::Type::static_type(),
                Some(&self_expression),
                "note",
            );
            let metadata_expression = gtk::PropertyExpression::new(
                Note::static_type(),
                Some(&note_expression),
                "metadata",
            );
            let last_modified_expression = gtk::PropertyExpression::new(
                Metadata::static_type(),
                Some(&metadata_expression),
                "last-modified",
            );
            let last_modified_str_expr = gtk::ClosureExpression::new(
                |args| {
                    let date: Date = args[1].get().unwrap();
                    // TODO use fuzzy here
                    format!("Last edited {}", date)
                },
                &[last_modified_expression.upcast()],
            );
            last_modified_str_expr.bind(
                &self.last_modified_label.get(),
                "label",
                None::<&gtk::Widget>,
            );
        }

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
}
