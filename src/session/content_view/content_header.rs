use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use crate::date::Date;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content_header.ui")]
    pub struct ContentHeader {
        #[template_child]
        pub title_label: TemplateChild<gtk::EditableLabel>,
        #[template_child]
        pub modified_label: TemplateChild<gtk::Label>,

        pub title: RefCell<String>,
        pub modified: RefCell<Date>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContentHeader {
        const NAME: &'static str = "NwtyContentHeader";
        type Type = super::ContentHeader;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContentHeader {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let this_expr = gtk::ConstantExpression::new(obj).upcast();
            let modified_expr = gtk::PropertyExpression::new(
                Self::Type::static_type(),
                Some(&this_expr),
                "modified",
            )
            .upcast();
            let modified_str_expr = gtk::ClosureExpression::new(
                |args| {
                    let date: Date = args[1].get().unwrap();
                    format!("Last edited {:?}", date)
                },
                &[modified_expr],
            );

            modified_str_expr.bind(&*self.modified_label, "label", None);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "title",
                        "Title",
                        "Title of the selected note",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boxed(
                        "modified",
                        "Modified",
                        "Last modified date of selected note",
                        Date::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "title" => {
                    let title = value.get().unwrap();
                    self.title.replace(title);
                }
                "modified" => {
                    let modified = value.get().unwrap();
                    self.modified.replace(modified);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "modified" => self.modified.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ContentHeader {}
    impl BoxImpl for ContentHeader {}
}

glib::wrapper! {
    pub struct ContentHeader(ObjectSubclass<imp::ContentHeader>)
        @extends gtk::Widget, gtk::Box;
}

impl ContentHeader {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ContentHeader.")
    }

    pub fn set_title(&self, title: &str) {
        self.set_property("title", title).unwrap();
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }
}
