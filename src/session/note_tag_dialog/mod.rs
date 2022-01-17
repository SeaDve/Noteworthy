mod note_tag_lists;
mod row;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone, closure},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use self::{note_tag_lists::NoteTagLists, row::Row};
use crate::model::{NoteTagList, Tag, TagList};

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

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
        pub other_tag_lists: OnceCell<NoteTagLists>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteTagDialog {
        const NAME: &'static str = "NwtyNoteTagDialog";
        type Type = super::NoteTagDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("note-tag-dialog.create-tag", None, move |obj, _, _| {
                obj.on_create_tag();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NoteTagDialog {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "tag-list",
                        "Tag List",
                        "List of tags",
                        TagList::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "other-tag-lists",
                        "List of other tag lists",
                        "The tag lists to compare with",
                        NoteTagLists::static_type(),
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
                "other-tag-lists" => {
                    let other_tag_lists = value.get().unwrap();
                    obj.set_other_tag_lists(other_tag_lists);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "tag-list" => obj.tag_list().to_value(),
                "other-tag-lists" => obj.other_tag_lists().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.action_set_enabled("note-tag-dialog.create-tag", false);

            obj.setup_list_view();
            obj.setup_signals();
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
    pub fn new(tag_list: &TagList, other_tag_lists: Vec<NoteTagList>) -> Self {
        glib::Object::new(&[
            ("tag-list", tag_list),
            ("other-tag-lists", &NoteTagLists::from(other_tag_lists)),
        ])
        .expect("Failed to create NoteTagDialog.")
    }

    fn other_tag_lists(&self) -> NoteTagLists {
        self.imp().other_tag_lists.get().unwrap().clone()
    }

    fn set_other_tag_lists(&self, other_tag_list: NoteTagLists) {
        self.imp().other_tag_lists.set(other_tag_list).unwrap();
    }

    fn tag_list(&self) -> TagList {
        self.imp().tag_list.get().unwrap().clone()
    }

    fn set_tag_list(&self, tag_list: TagList) {
        let imp = self.imp();

        let tag_name_expression = gtk::ClosureExpression::new::<String, &[gtk::Expression], _>(
            &[],
            closure!(|tag: Tag| tag.name()),
        );
        let filter = gtk::StringFilter::builder()
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

    fn on_create_tag(&self) {
        let imp = self.imp();
        let tag_name = imp.search_entry.text();
        let new_tag = Tag::new(&tag_name);

        self.other_tag_lists().append_on_all(&new_tag);

        self.tag_list().append(new_tag).unwrap();
        // TODO new_tag should be added on top

        imp.search_entry.set_text("");
    }

    fn on_search_entry_text_notify(&self, tag_name: &str) {
        let does_contain_tag = self.tag_list().contains_with_name(tag_name);

        let is_create_tag_enabled = !does_contain_tag && !tag_name.is_empty();
        self.action_set_enabled("note-tag-dialog.create-tag", is_create_tag_enabled);

        let imp = self.imp();
        imp.create_tag_button_revealer
            .set_reveal_child(is_create_tag_enabled);
        imp.create_tag_button_label
            .set_label(&gettext!("Create “{}”", tag_name));
    }

    fn setup_list_view(&self) {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(clone!(@weak self as obj => move |_, list_item| {
            let tag_row = Row::new(&obj.other_tag_lists());

            list_item
                .property_expression("item")
                .bind(&tag_row, "tag", glib::Object::NONE);

            list_item.set_child(Some(&tag_row));
        }));

        self.imp().list_view.set_factory(Some(&factory));
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        imp.search_entry
            .connect_text_notify(clone!(@weak self as obj => move |search_entry| {
                obj.on_search_entry_text_notify(&search_entry.text());
            }));

        imp.search_entry
            .connect_activate(clone!(@weak self as obj => move |_| {
                WidgetExt::activate_action(&obj, "note-tag-dialog.create-tag", None).unwrap();
            }));
    }
}
