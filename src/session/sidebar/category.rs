use gtk::{gio, glib, glib::clone, prelude::*, subclass::prelude::*};
use once_cell::unsync::OnceCell;

use std::cell::RefCell;

use super::{Note, NoteList};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Category {
        pub display_name: RefCell<String>,
        pub filter: OnceCell<gtk::Filter>,
        pub model: OnceCell<gio::ListModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Category {
        const NAME: &'static str = "NwtyCategory";
        type Type = super::Category;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for Category {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "display-name",
                        "Display Name",
                        "Display name of this category",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "filter",
                        "Filter",
                        "Filters the model",
                        gtk::Filter::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "model",
                        "Model",
                        "The filter list model in that category",
                        gio::ListModel::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "display-name" => {
                    let display_name = value.get().unwrap();
                    self.display_name.replace(display_name);
                }
                "filter" => {
                    let filter = value.get().unwrap();
                    self.filter.set(filter).unwrap();
                }
                "model" => {
                    let model = value.get().unwrap();
                    obj.set_model(model);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "display-name" => self.display_name.borrow().to_value(),
                "filter" => self.filter.get().to_value(),
                "model" => self.model.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for Category {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Note::static_type()
        }
        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.model.get().map_or(0, |l| l.n_items())
        }
        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.model.get().and_then(|l| l.item(position))
        }
    }
}

glib::wrapper! {
    pub struct Category(ObjectSubclass<imp::Category>)
        @implements gio::ListModel;
}

impl Category {
    pub fn new(title: &str, filter: &gtk::Filter, model: &NoteList) -> Self {
        glib::Object::new(&[
            ("display-name", &title.to_string()),
            ("filter", filter),
            ("model", model),
        ])
        .expect("Failed to create Category")
    }

    fn set_model(&self, model: gio::ListModel) {
        let imp = imp::Category::from_instance(self);
        let filter = imp.filter.get().unwrap();

        let filter_model = gtk::FilterListModel::new(Some(&model), Some(filter));

        let sorter = gtk::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<Note>().unwrap().metadata();
            let b = b.downcast_ref::<Note>().unwrap().metadata();

            b.last_modified().cmp(&a.last_modified()).into()
        });
        let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));

        sort_model.connect_items_changed(
            clone!(@weak self as obj => move |_, pos, added, removed| {
                obj.items_changed(pos, added, removed);
            }),
        );

        imp.model.set(sort_model.upcast()).unwrap();
    }
}
