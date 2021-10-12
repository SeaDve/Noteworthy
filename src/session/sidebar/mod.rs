mod note_row;
mod selection;
mod view_switcher;

use gettextrs::gettext;
use gtk::{
    glib::{self, clone, GEnum},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::{Cell, RefCell};

use self::{
    note_row::NoteRow,
    selection::Selection,
    view_switcher::{ItemKind, ViewSwitcher},
};
use super::{tag_list::TagList, Note, NoteList};
use crate::components::SyncButton;

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

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar.ui")]
    pub struct Sidebar {
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub view_switcher: TemplateChild<ViewSwitcher>,
        #[template_child]
        pub header_bar_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub main_header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub selection_header_bar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub selection_menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub action_bar: TemplateChild<gtk::ActionBar>,
        #[template_child]
        pub pin_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub trash_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub tag_button: TemplateChild<gtk::Button>,

        pub compact: Cell<bool>,
        pub selection_mode: Cell<SelectionMode>,
        pub selected_note: RefCell<Option<Note>>,

        pub single_selection_model: RefCell<Option<Selection>>,
        pub multi_selection_model: RefCell<Option<gtk::MultiSelection>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "NwtySidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            ViewSwitcher::static_type();
            NoteRow::static_type();
            SyncButton::static_type();
            Self::bind_template(klass);

            klass.install_action(
                "sidebar.multi-selection-mode-done",
                None,
                move |obj, _, _| {
                    obj.set_selection_mode(SelectionMode::Single);
                },
            );

            klass.install_action("sidebar.select-all", None, move |obj, _, _| {
                let model = obj.multi_selection_model().unwrap();
                model.select_all();
            });

            klass.install_action("sidebar.select-none", None, move |obj, _, _| {
                let model = obj.multi_selection_model().unwrap();
                model.unselect_all();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Sidebar {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_list_view();
            obj.setup_expressions();
            obj.setup_signals();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boolean(
                        "compact",
                        "Compact",
                        "Whether it is compact view mode",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_enum(
                        "selection-mode",
                        "Selection Mode",
                        "Current selection mode",
                        SelectionMode::static_type(),
                        SelectionMode::default() as i32,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "note-list",
                        "Note List",
                        "Note list represented by self",
                        NoteList::static_type(),
                        glib::ParamFlags::WRITABLE,
                    ),
                    glib::ParamSpec::new_object(
                        "selected-note",
                        "Selected Note",
                        "The selected note in this sidebar",
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
                "selection-mode" => {
                    let selection_mode = value.get().unwrap();
                    obj.set_selection_mode(selection_mode);
                }
                "note-list" => {
                    let note_list = value.get().unwrap();
                    obj.set_note_list(note_list);
                }
                "selected-note" => {
                    let selected_note = value.get().unwrap();
                    obj.set_selected_note(selected_note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "selection-mode" => obj.selection_mode().to_value(),
                "selected-note" => obj.selected_note().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Sidebar {}
    impl BoxImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget, gtk::Box;
}

impl Sidebar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Sidebar.")
    }

    pub fn set_note_list(&self, note_list: NoteList) {
        let imp = imp::Sidebar::from_instance(self);

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let note_1 = obj1.downcast_ref::<Note>().unwrap().metadata();
            let note_2 = obj2.downcast_ref::<Note>().unwrap().metadata();

            // Sort is pinned first before classifying by last modified
            if note_1.is_pinned() == note_2.is_pinned() {
                note_2.last_modified().cmp(&note_1.last_modified()).into()
            } else if note_1.is_pinned() && !note_2.is_pinned() {
                gtk::Ordering::Smaller
            } else {
                gtk::Ordering::Larger
            }
        });
        let sorter_model = gtk::SortListModel::new(Some(&note_list), Some(&sorter));

        let filter_expression = gtk::ClosureExpression::new(
            clone!(@weak self as obj => @default-return true, move |value| {
                let note = value[0].get::<Note>().unwrap().metadata();
                let imp = imp::Sidebar::from_instance(&obj);

                match imp.view_switcher.selected_type() {
                    ItemKind::AllNotes => !note.is_trashed(),
                    ItemKind::Trash => note.is_trashed(),
                    ItemKind::Tag(ref tag) => {
                        note.tag_list().contains(tag) && !note.is_trashed()
                    }
                    ItemKind::Separator | ItemKind::Category | ItemKind::EditTags => {
                        // FIXME handle this inside view_switcher
                        log::warn!("Trying to select an unselectable row");
                        imp.view_switcher.set_selected_item_to_default();
                        true
                    }
                }
            }),
            &[],
        );
        let filter = gtk::BoolFilterBuilder::new()
            .expression(&filter_expression)
            .build();
        let filter_model = gtk::FilterListModel::new(Some(&sorter_model), Some(&filter));

        imp.view_switcher.connect_selected_type_notify(move |_, _| {
            filter.changed(gtk::FilterChange::Different);
        });

        let single_selection_model = Selection::new(Some(&filter_model));
        self.bind_property("selected-note", &single_selection_model, "selected-item")
            .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
            .build();
        imp.single_selection_model
            .replace(Some(single_selection_model));

        let multi_selection_model = gtk::MultiSelection::new(Some(&filter_model));
        multi_selection_model.connect_selection_changed(
            clone!(@weak self as obj => move |model,_,_| {
                let selection_size = model.selection().size();
                obj.update_selection_menu_button_label(selection_size);
                obj.update_action_bar_sensitivity(selection_size);
                obj.update_action_bar(model);
                log::info!("Selection changed, n_items: {}", selection_size);
            }),
        );
        multi_selection_model.connect_items_changed(
            clone!(@weak self as obj => move |model,pos,removed,added| {
                let selection_size = model.selection().size();
                obj.update_selection_menu_button_label(selection_size);
                obj.update_action_bar_sensitivity(selection_size);
                obj.update_action_bar(model);
                log::info!("Selection items changed at {}; {} removed, {} added", pos, removed, added);
            }),
        );
        imp.multi_selection_model
            .replace(Some(multi_selection_model));

        self.set_selection_mode(SelectionMode::Single);
    }

    pub fn set_selected_note(&self, selected_note: Option<Note>) {
        if self.selected_note() == selected_note {
            return;
        }

        let imp = imp::Sidebar::from_instance(self);
        imp.selected_note.replace(selected_note);
        self.notify("selected-note");
    }

    pub fn selected_note(&self) -> Option<Note> {
        let imp = imp::Sidebar::from_instance(self);
        imp.selected_note.borrow().clone()
    }

    pub fn set_tag_list(&self, tag_list: TagList) {
        let imp = imp::Sidebar::from_instance(self);
        imp.view_switcher.set_tag_list(tag_list);
    }

    pub fn selection_mode(&self) -> SelectionMode {
        let imp = imp::Sidebar::from_instance(self);
        imp.selection_mode.get()
    }

    pub fn set_selection_mode(&self, selection_mode: SelectionMode) {
        let imp = imp::Sidebar::from_instance(self);

        // FIXME just use one model so scroll level will be saved
        match selection_mode {
            SelectionMode::Single => {
                imp.header_bar_stack
                    .set_visible_child(&imp.main_header_bar.get());
                imp.action_bar.set_revealed(false);

                let model = imp.single_selection_model.borrow();
                imp.list_view.set_model(Some(model.as_ref().unwrap()));
                imp.list_view.set_single_click_activate(true);
                imp.list_view
                    .remove_css_class("sidebar-list-view-multi-selection-mode");
            }
            SelectionMode::Multi => {
                imp.header_bar_stack
                    .set_visible_child(&imp.selection_header_bar.get());
                imp.action_bar.set_revealed(true);

                let model = imp.multi_selection_model.borrow();
                imp.list_view.set_model(Some(model.as_ref().unwrap()));
                imp.list_view.set_single_click_activate(false);
                imp.list_view
                    .add_css_class("sidebar-list-view-multi-selection-mode");
            }
        }

        imp.selection_mode.set(selection_mode);
        self.notify("selection-mode");
    }

    pub fn multi_selection_model(&self) -> Option<gtk::MultiSelection> {
        let imp = imp::Sidebar::from_instance(self);
        imp.multi_selection_model.borrow().as_ref().cloned()
    }

    // FIXME merge this with selected_notes()
    pub fn selected_notes_bitset(&self) -> gtk::Bitset {
        let model = self.multi_selection_model().unwrap();
        model.selection()
    }

    // FIXME make this an iterator to not iterate twice
    pub fn selected_notes(&self) -> Vec<Note> {
        let model = self.multi_selection_model().unwrap();
        let bitset = model.selection();
        let mut note_vec = Vec::new();

        if let Some((bitset_iter, index)) = gtk::BitsetIter::init_first(&bitset) {
            note_vec.push(model.item(index).unwrap().downcast::<Note>().unwrap());
            for index in bitset_iter {
                note_vec.push(model.item(index).unwrap().downcast::<Note>().unwrap());
            }
        }

        note_vec
    }

    fn update_action_bar_sensitivity(&self, n_selected_items: u64) {
        let is_selection_empty = n_selected_items == 0;

        let imp = imp::Sidebar::from_instance(self);
        imp.tag_button.set_sensitive(!is_selection_empty);
        imp.trash_button.set_sensitive(!is_selection_empty);
        imp.pin_button.set_sensitive(!is_selection_empty);
    }

    fn update_selection_menu_button_label(&self, n_selected_items: u64) {
        let imp = imp::Sidebar::from_instance(self);
        let label = if n_selected_items == 0 {
            gettext("No selected")
        } else {
            gettext!("{} selected", n_selected_items)
        };
        imp.selection_menu_button.set_label(&label);
    }

    fn update_action_bar(&self, model: &gtk::MultiSelection) {
        let imp = imp::Sidebar::from_instance(self);

        let is_all_pinned_in_selected_notes = {
            // Just check the last selected note to short circuit faster
            // since the selection is always sorted pinned first. Therefore, for all selected notes
            // to be all pinned, the last one has to be pinned.
            gtk::BitsetIter::init_last(&self.selected_notes_bitset())
                .and_then(|(_, index)| model.item(index))
                .map_or(false, |item| {
                    let note = item.downcast::<Note>().unwrap();
                    note.metadata().is_pinned()
                })
        };

        imp.pin_button.set_active(is_all_pinned_in_selected_notes);

        // It is only possible for trash button to be active when we are on trash page
        let is_on_trash_page = imp.view_switcher.selected_type() == ItemKind::Trash;
        let is_selection_empty = model.selection().size() == 0;
        imp.trash_button
            .set_active(is_on_trash_page && !is_selection_empty);
    }

    fn setup_signals(&self) {
        let imp = imp::Sidebar::from_instance(self);

        imp.trash_button
            .connect_clicked(clone!(@weak self as obj => move |button| {
                let is_active = button.is_active();
                for note in obj.selected_notes().iter() {
                    note.metadata().set_is_trashed(is_active);
                }
            }));

        imp.pin_button
            .connect_clicked(clone!(@weak self as obj => move |button| {
                let is_active = button.is_active();
                for note in obj.selected_notes().iter() {
                    note.metadata().set_is_pinned(is_active);
                }
            }));
    }

    fn setup_expressions(&self) {
        let imp = imp::Sidebar::from_instance(self);

        let list_view_expression = gtk::ConstantExpression::new(&imp.list_view.get());
        let model_expression = gtk::PropertyExpression::new(
            gtk::ListView::static_type(),
            Some(&list_view_expression),
            "model",
        );
        let model_is_some_expression = gtk::ClosureExpression::new(
            |args| {
                let model: Option<gtk::SelectionModel> = args[1].get().unwrap();

                if model.is_some() {
                    "filled-view"
                } else {
                    "empty-view"
                }
            },
            &[model_expression.upcast()],
        );
        model_is_some_expression.bind(&imp.stack.get(), "visible-child-name", None::<&gtk::Widget>);
    }

    fn setup_list_view(&self) {
        let imp = imp::Sidebar::from_instance(self);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(clone!(@weak self as obj => move |_, list_item| {
            let note_row = NoteRow::new(&obj);
            obj.bind_property("selection-mode", &note_row, "selection-mode").flags(glib::BindingFlags::SYNC_CREATE).build();

            let list_item_expression = gtk::ConstantExpression::new(list_item);

            let note_expression = gtk::PropertyExpression::new(
                gtk::ListItem::static_type(),
                Some(&list_item_expression),
                "item",
            );
            note_expression.bind(&note_row, "note", None::<&gtk::Widget>);

            let selected_expression = gtk::PropertyExpression::new(
                gtk::ListItem::static_type(),
                Some(&list_item_expression),
                "selected",
            );
            selected_expression.bind(&note_row, "is-checked", None::<&gtk::Widget>);

            let position_expression = gtk::PropertyExpression::new(
                gtk::ListItem::static_type(),
                Some(&list_item_expression),
                "position"
            );
            position_expression.bind(&note_row, "position", None::<&gtk::Widget>);

            list_item.set_child(Some(&note_row));
        }));

        imp.list_view.set_factory(Some(&factory));

        imp.list_view
            .get()
            .connect_activate(move |list_view, index| {
                let model: Option<Selection> = list_view.model().and_then(|o| o.downcast().ok());
                let note: Option<glib::Object> = model.as_ref().and_then(|m| m.item(index));

                if let (Some(model), Some(_)) = (model, note) {
                    model.set_selected(index);
                }
            });
    }
}
