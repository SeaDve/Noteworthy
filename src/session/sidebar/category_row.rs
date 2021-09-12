use adw::subclass::prelude::BinImpl;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use super::category::Category;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar_category_row.ui")]
    pub struct CategoryRow {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        pub category: RefCell<Option<Category>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CategoryRow {
        const NAME: &'static str = "NwtySidebarCategoryRow";
        type Type = super::CategoryRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CategoryRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "category",
                    "Category",
                    "The category of this row",
                    Category::static_type(),
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
                "category" => {
                    let category = value.get().unwrap();
                    obj.set_category(category);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "category" => obj.category().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for CategoryRow {}
    impl BinImpl for CategoryRow {}
}

glib::wrapper! {
    pub struct CategoryRow(ObjectSubclass<imp::CategoryRow>)
        @extends gtk::Widget, adw::Bin, @implements gtk::Accessible;
}

impl CategoryRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create CategoryRow")
    }

    pub fn category(&self) -> Option<Category> {
        let imp = imp::CategoryRow::from_instance(self);
        imp.category.borrow().clone()
    }

    pub fn set_category(&self, category: Option<Category>) {
        let imp = imp::CategoryRow::from_instance(self);

        imp.category.replace(category);
        self.notify("category");
    }
}
