// Class taken from Fractal-next
// See https://gitlab.gnome.org/GNOME/fractal/-/blob/fractal-next/src/session/sidebar/selection.rs
// This file is modifed to change between selection modes: Single and Multiple.

use gtk::{
    gio,
    glib::{self, clone, GEnum},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::sync::Lazy;

use std::cell::{Cell, RefCell};

#[derive(Debug, Clone, Copy, PartialEq, GEnum)]
#[genum(type_name = "SidebarSelectionMode")]
pub enum SelectionMode {
    Single,
    Multi,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::Single
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Selection {
        pub model: RefCell<Option<gio::ListModel>>,
        pub selected: Cell<u32>,
        pub selected_item: RefCell<Option<glib::Object>>,
        pub selection_mode: Cell<SelectionMode>,

        pub multi_selection_model: RefCell<Option<gtk::MultiSelection>>,
        pub model_items_changed_id: RefCell<Option<glib::SignalHandlerId>>,
        pub multi_model_items_changed_id: RefCell<Option<glib::SignalHandlerId>>,
        pub multi_model_selection_changed_id: RefCell<Option<glib::SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Selection {
        const NAME: &'static str = "NwtySidebarSelection";
        type Type = super::Selection;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel, gtk::SelectionModel);

        fn new() -> Self {
            Self {
                selected: Cell::new(gtk::INVALID_LIST_POSITION),
                ..Default::default()
            }
        }
    }

    impl ObjectImpl for Selection {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "model",
                        "Model",
                        "The model being managed",
                        gio::ListModel::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_uint(
                        "selected",
                        "Selected",
                        "The position of the selected item",
                        0,
                        u32::MAX,
                        gtk::INVALID_LIST_POSITION,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-item",
                        "Selected Item",
                        "The selected item",
                        glib::Object::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_enum(
                        "selection-mode",
                        "Selection Mode",
                        "Current selection mode",
                        SelectionMode::static_type(),
                        SelectionMode::default() as i32,
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
                "model" => {
                    let model: Option<gio::ListModel> = value.get().unwrap();
                    obj.set_model(model.as_ref());
                }
                "selected" => {
                    let selected = value.get().unwrap();
                    obj.set_selected(selected);
                }
                "selected-item" => {
                    let selected_item = value.get().unwrap();
                    obj.set_selected_item(selected_item);
                }
                "selection-mode" => {
                    let selection_mode = value.get().unwrap();
                    obj.set_selection_mode(selection_mode);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "model" => obj.model().to_value(),
                "selected" => obj.selected().to_value(),
                "selected-item" => obj.selected_item().to_value(),
                "selection-mode" => obj.selection_mode().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for Selection {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            glib::Object::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.model.borrow().as_ref().map_or(0, |m| m.n_items())
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.model.borrow().as_ref().and_then(|m| m.item(position))
        }
    }

    impl SelectionModelImpl for Selection {
        fn selection_in_range(&self, obj: &Self::Type, position: u32, n_items: u32) -> gtk::Bitset {
            if obj.selection_mode() == SelectionMode::Multi {
                return obj
                    .multi_selection_model()
                    .selection_in_range(position, n_items);
            }

            let bitset = gtk::Bitset::new_empty();
            let selected = self.selected.get();

            if selected != gtk::INVALID_LIST_POSITION {
                bitset.add(selected);
            }

            bitset
        }

        fn is_selected(&self, obj: &Self::Type, position: u32) -> bool {
            if obj.selection_mode() == SelectionMode::Multi {
                return obj.multi_selection_model().is_selected(position);
            }

            self.selected.get() == position
        }

        fn set_selection(
            &self,
            obj: &Self::Type,
            selected: &gtk::Bitset,
            mask: &gtk::Bitset,
        ) -> bool {
            if obj.selection_mode() == SelectionMode::Multi {
                return obj.multi_selection_model().set_selection(selected, mask);
            }

            true
        }
    }
}

glib::wrapper! {
    pub struct Selection(ObjectSubclass<imp::Selection>)
        @implements gio::ListModel, gtk::SelectionModel;
}

impl Selection {
    pub fn new<P: IsA<gio::ListModel>>(model: Option<&P>) -> Selection {
        let model = model.map(|m| m.clone().upcast::<gio::ListModel>());
        glib::Object::new(&[("model", &model)]).expect("Failed to create Selection")
    }

    pub fn model(&self) -> Option<gio::ListModel> {
        let imp = imp::Selection::from_instance(self);
        imp.model.borrow().clone()
    }

    pub fn selected(&self) -> u32 {
        let imp = imp::Selection::from_instance(self);
        imp.selected.get()
    }

    pub fn selected_item(&self) -> Option<glib::Object> {
        let imp = imp::Selection::from_instance(self);
        imp.selected_item.borrow().clone()
    }

    pub fn selection_mode(&self) -> SelectionMode {
        let imp = imp::Selection::from_instance(self);
        imp.selection_mode.get()
    }

    pub fn set_selection_mode(&self, selection_mode: SelectionMode) {
        let imp = imp::Selection::from_instance(self);
        imp.selection_mode.set(selection_mode);
        self.notify("selection-mode");

        let selected = self.selected();
        if selected != gtk::INVALID_LIST_POSITION {
            self.selection_changed(selected, 1);
        }

        let multi_selection = self.multi_selection_model().selection();
        let min = multi_selection.minimum();
        let max = multi_selection.maximum();

        if min <= max {
            self.selection_changed(min, max - min + 1);
        }
    }

    pub fn set_model<P: IsA<gio::ListModel>>(&self, model: Option<&P>) {
        let imp = imp::Selection::from_instance(self);

        let _guard = self.freeze_notify();

        let model = model.map(|m| m.clone().upcast::<gio::ListModel>());

        let old_model = self.model();
        if old_model == model {
            return;
        }

        if let Some(id) = imp.multi_model_items_changed_id.take() {
            let old_model = self.multi_selection_model();
            old_model.disconnect(id);
        }

        if let Some(id) = imp.multi_model_selection_changed_id.take() {
            let old_model = self.multi_selection_model();
            old_model.disconnect(id);
        }

        let n_items_before = old_model.map_or(0, |model| {
            if let Some(id) = imp.model_items_changed_id.take() {
                model.disconnect(id);
            }
            model.n_items()
        });

        if let Some(model) = model {
            let model_items_changed_id =
                model.connect_items_changed(clone!(@weak self as obj => move |m, p, r, a| {
                    if obj.selection_mode() == SelectionMode::Single {
                        obj.items_changed_cb(m, p, r, a);
                    }
                }));
            imp.model_items_changed_id
                .replace(Some(model_items_changed_id));

            let multi_selection_model = gtk::MultiSelection::new(Some(&model));

            let multi_model_items_changed_id = multi_selection_model.connect_items_changed(
                clone!(@weak self as obj => move |_, p, r, a| {
                    if obj.selection_mode() == SelectionMode::Multi {
                        obj.items_changed(p, r, a);
                    }
                }),
            );
            imp.multi_model_items_changed_id
                .replace(Some(multi_model_items_changed_id));

            let multi_model_selection_changed_id = multi_selection_model.connect_selection_changed(
                clone!(@weak self as obj => move |_, position, n_items| {
                    if obj.selection_mode() == SelectionMode::Multi {
                        obj.selection_changed(position, n_items);
                    }
                }),
            );
            imp.multi_model_selection_changed_id
                .replace(Some(multi_model_selection_changed_id));

            self.items_changed_cb(&model, 0, n_items_before, model.n_items());

            imp.model.replace(Some(model));
            imp.multi_selection_model
                .replace(Some(multi_selection_model));
        } else {
            imp.model.replace(None);

            if self.selected() != gtk::INVALID_LIST_POSITION {
                imp.selected.replace(gtk::INVALID_LIST_POSITION);
                self.notify("selected");
            }
            if self.selected_item().is_some() {
                imp.selected_item.replace(None);
                self.notify("selected-item");
            }

            self.items_changed(0, n_items_before, 0);
        }

        self.notify("model");
    }

    pub fn set_selected(&self, position: u32) {
        let imp = imp::Selection::from_instance(self);

        let old_selected = self.selected();
        if old_selected == position {
            return;
        }

        let selected_item = self.model().and_then(|m| m.item(position));

        let selected = if selected_item.is_none() {
            gtk::INVALID_LIST_POSITION
        } else {
            position
        };

        if old_selected == selected {
            return;
        }

        imp.selected.replace(selected);
        imp.selected_item.replace(selected_item);

        if old_selected == gtk::INVALID_LIST_POSITION {
            self.selection_changed(selected, 1);
        } else if selected == gtk::INVALID_LIST_POSITION {
            self.selection_changed(old_selected, 1);
        } else if selected < old_selected {
            self.selection_changed(selected, old_selected - selected + 1);
        } else {
            self.selection_changed(old_selected, selected - old_selected + 1);
        }

        self.notify("selected");
        self.notify("selected-item");
    }

    fn multi_selection_model(&self) -> gtk::MultiSelection {
        let imp = imp::Selection::from_instance(self);
        imp.multi_selection_model
            .borrow()
            .as_ref()
            .expect("Multi selection model not set")
            .clone()
    }

    fn set_selected_item(&self, item: Option<glib::Object>) {
        let imp = imp::Selection::from_instance(self);

        let selected_item = self.selected_item();
        if selected_item == item {
            return;
        }

        let old_selected = self.selected();

        let mut selected = gtk::INVALID_LIST_POSITION;

        if item.is_some() {
            if let Some(model) = self.model() {
                for i in 0..model.n_items() {
                    let current_item = model.item(i);
                    if current_item == item {
                        selected = i;
                        break;
                    }
                }
            }
        }

        imp.selected_item.replace(item);

        if old_selected != selected {
            imp.selected.replace(selected);

            if old_selected == gtk::INVALID_LIST_POSITION {
                self.selection_changed(selected, 1);
            } else if selected == gtk::INVALID_LIST_POSITION {
                self.selection_changed(old_selected, 1);
            } else if selected < old_selected {
                self.selection_changed(selected, old_selected - selected + 1);
            } else {
                self.selection_changed(old_selected, selected - old_selected + 1);
            }
            self.notify("selected");
        }

        self.notify("selected-item");
    }

    fn items_changed_cb(&self, model: &gio::ListModel, position: u32, removed: u32, added: u32) {
        let imp = imp::Selection::from_instance(self);

        let _guard = self.freeze_notify();

        let selected = self.selected();
        let selected_item = self.selected_item();

        if selected_item.is_none() || selected < position {
            // unchanged
        } else if selected != gtk::INVALID_LIST_POSITION && selected >= position + removed {
            imp.selected.replace(selected + added - removed);
            self.notify("selected");
        } else {
            for i in 0..=added {
                if i == added {
                    // the item really was deleted
                    imp.selected.replace(gtk::INVALID_LIST_POSITION);
                    self.notify("selected");
                } else {
                    let item = model.item(position + i);
                    if item == selected_item {
                        // the item moved
                        if selected != position + i {
                            imp.selected.replace(position + i);
                            self.notify("selected");
                        }
                        break;
                    }
                }
            }
        }

        self.items_changed(position, removed, added);
    }
}
