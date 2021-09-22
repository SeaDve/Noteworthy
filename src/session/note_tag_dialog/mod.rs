mod row;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::unsync::OnceCell;

use self::row::Row;
use super::{note::NoteTagList, tag::Tag, tag_list::TagList};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/note-tag-dialog.ui")]
    pub struct NoteTagDialog {
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub create_tag_button_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub create_tag_button_label: TemplateChild<gtk::Label>,

        pub tag_list: OnceCell<TagList>,
        pub other_tag_list: OnceCell<NoteTagList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteTagDialog {
        const NAME: &'static str = "NwtyNoteTagDialog";
        type Type = super::NoteTagDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Row::static_type();
            Self::bind_template(klass);

            klass.install_action("note-tag-dialog.create-tag", None, move |obj, _, _| {
                let imp = imp::NoteTagDialog::from_instance(obj);
                let tag_name = imp.search_entry.text();
                let new_tag = Tag::new(&tag_name);

                obj.other_tag_list().append(new_tag.clone()).unwrap();
                obj.tag_list().append(new_tag).unwrap();
                // TODO new_tag should be added on top

                imp.search_entry.set_text("");
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NoteTagDialog {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "tag-list",
                        "Tag List",
                        "List of tags",
                        TagList::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "other-tag-list",
                        "Other Tag List",
                        "The list of tags to compare with",
                        NoteTagList::static_type(),
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
                "tag-list" => {
                    let tag_list = value.get().unwrap();
                    obj.set_tag_list(tag_list);
                }
                "other-tag-list" => {
                    let other_tag_list = value.get().unwrap();
                    obj.set_other_tag_list(other_tag_list);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "tag-list" => self.tag_list.get().to_value(),
                "other-tag-list" => self.other_tag_list.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(clone!(@weak obj => move |_, list_item| {
                let tag_row = Row::new(&obj.other_tag_list());

                let list_item_expression = gtk::ConstantExpression::new(list_item);
                let item_expression = gtk::PropertyExpression::new(
                    gtk::ListItem::static_type(),
                    Some(&list_item_expression),
                    "item",
                );
                item_expression.bind(&tag_row, "tag", None::<&gtk::Widget>);

                list_item.set_child(Some(&tag_row));
            }));
            self.list_view.set_factory(Some(&factory));

            self.search_entry.connect_text_notify(
                clone!(@weak obj => move |search_entry| {
                    let search_entry_text = search_entry.text();
                    let does_contain_tag = obj.tag_list().contains_with_name(&search_entry_text);
                    let is_search_entry_empty = search_entry_text.is_empty();
                    let imp = imp::NoteTagDialog::from_instance(&obj);
                    imp.create_tag_button_revealer.set_reveal_child(!does_contain_tag && !is_search_entry_empty);
                    imp.create_tag_button_label.set_label(&gettext!("Create “{}”", search_entry_text));
                }),
            );
        }
    }

    impl WidgetImpl for NoteTagDialog {}
    impl WindowImpl for NoteTagDialog {}
    impl AdwWindowImpl for NoteTagDialog {}
}

glib::wrapper! {
    pub struct NoteTagDialog(ObjectSubclass<imp::NoteTagDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl NoteTagDialog {
    pub fn new(tag_list: &TagList, other_tag_list: &NoteTagList) -> Self {
        glib::Object::new(&[("tag-list", tag_list), ("other-tag-list", other_tag_list)])
            .expect("Failed to create NoteTagDialog.")
    }

    fn other_tag_list(&self) -> NoteTagList {
        let imp = imp::NoteTagDialog::from_instance(self);
        imp.other_tag_list.get().unwrap().clone()
    }

    fn set_other_tag_list(&self, other_tag_list: NoteTagList) {
        let imp = imp::NoteTagDialog::from_instance(self);
        imp.other_tag_list.set(other_tag_list).unwrap();
    }

    fn tag_list(&self) -> TagList {
        let imp = imp::NoteTagDialog::from_instance(self);
        imp.tag_list.get().unwrap().clone()
    }

    fn set_tag_list(&self, tag_list: TagList) {
        let imp = imp::NoteTagDialog::from_instance(self);

        let tag_name_expression =
            gtk::ClosureExpression::new(|value| value[0].get::<Tag>().unwrap().name(), &[]);
        let filter = gtk::StringFilterBuilder::new()
            .match_mode(gtk::StringFilterMatchMode::Substring)
            .expression(&tag_name_expression)
            .ignore_case(true)
            .build();
        let filter_model = gtk::FilterListModel::new(Some(&tag_list), Some(&filter));

        imp.search_entry
            .bind_property("text", &filter, "search")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        let selection_model = gtk::NoSelection::new(Some(&filter_model));
        imp.list_view.set_model(Some(&selection_model));

        imp.tag_list.set(tag_list).unwrap();
    }
}
