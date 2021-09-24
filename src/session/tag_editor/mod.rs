mod row;

use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::unsync::OnceCell;

use self::row::Row;
use super::{tag::Tag, tag_list::TagList, NoteList};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/tag-editor.ui")]
    pub struct TagEditor {
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub create_tag_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub create_tag_button: TemplateChild<gtk::Button>,

        pub tag_list: OnceCell<TagList>,
        pub note_list: OnceCell<NoteList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TagEditor {
        const NAME: &'static str = "NwtyTagEditor";
        type Type = super::TagEditor;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Row::static_type();
            Self::bind_template(klass);

            klass.install_action("tag-editor.create-tag", None, move |obj, _, _| {
                let imp = imp::TagEditor::from_instance(obj);
                let name = imp.create_tag_entry.text();

                let tag_list = obj.tag_list();
                tag_list.append(Tag::new(&name)).unwrap();

                imp.create_tag_entry.set_text("");
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TagEditor {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "tag-list",
                        "Tag List",
                        "List of tags",
                        TagList::static_type(),
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "note-list",
                        "Note List",
                        "List of notes",
                        NoteList::static_type(),
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "note-list" => {
                    let note_list = value.get().unwrap();
                    obj.set_note_list(note_list);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "tag-list" => obj.tag_list().to_value(),
                "note-list" => obj.note_list().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.create_tag_button.set_sensitive(false);
            self.create_tag_entry
                .connect_text_notify(clone!(@weak obj => move |entry| {
                    let imp = imp::TagEditor::from_instance(&obj);
                    let text = entry.text();

                    if obj.tag_list().is_valid_name(&text) {
                        imp.create_tag_button.set_sensitive(true);
                        imp.create_tag_entry.remove_css_class("error");
                    } else {
                        imp.create_tag_button.set_sensitive(false);
                        imp.create_tag_entry.add_css_class("error");
                    }
                }));
        }
    }

    impl WidgetImpl for TagEditor {}
    impl WindowImpl for TagEditor {}
    impl AdwWindowImpl for TagEditor {}
}

glib::wrapper! {
    pub struct TagEditor(ObjectSubclass<imp::TagEditor>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl TagEditor {
    pub fn new(tag_list: &TagList, note_list: &NoteList) -> Self {
        glib::Object::new(&[("tag-list", tag_list), ("note-list", note_list)])
            .expect("Failed to create TagEditor.")
    }

    pub fn tag_list(&self) -> TagList {
        let imp = imp::TagEditor::from_instance(self);
        imp.tag_list.get().unwrap().clone()
    }

    pub fn note_list(&self) -> NoteList {
        let imp = imp::TagEditor::from_instance(self);
        imp.note_list.get().unwrap().clone()
    }

    fn set_tag_list(&self, tag_list: TagList) {
        let imp = imp::TagEditor::from_instance(self);

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

    fn set_note_list(&self, note_list: NoteList) {
        let imp = imp::TagEditor::from_instance(self);
        imp.note_list.set(note_list).unwrap();
    }
}
