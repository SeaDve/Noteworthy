use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::RefCell;

use super::{Tag, TagList};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/tag-dialog-row.ui")]
    pub struct Row {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub check_button: TemplateChild<gtk::CheckButton>,

        pub other_tag_list: RefCell<TagList>,
        pub tag: RefCell<Option<Tag>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "NwtyTagDialogRow";
        type Type = super::Row;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Row {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "other-tag-list",
                        "Other Tag List",
                        "The tag list to compare with",
                        TagList::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "tag",
                        "tag",
                        "The tag represented by this row",
                        Tag::static_type(),
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
                "other-tag-list" => {
                    let other_tag_list = value.get().unwrap();
                    self.other_tag_list.replace(other_tag_list);
                }
                "tag" => {
                    let tag = value.get().unwrap();
                    self.tag.replace(tag);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "other-tag-list" => self.other_tag_list.borrow().to_value(),
                "tag" => self.tag.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // TODO Implement this so clicking the row activates the checkbutton
            // Works well when clicking the row but when you click the button it gets activated
            // twice, Idk how to not let the click pass through both widgets
            // let gesture = gtk::GestureClick::new();
            // gesture.connect_pressed(clone!(@weak obj => move |_,_,_,_| {
            //     let imp = imp::Row::from_instance(&obj);
            //     imp.check_button.activate();
            // }));
            // obj.add_controller(&gesture);

            let self_expression = gtk::ConstantExpression::new(&obj);
            let tag_expression = gtk::PropertyExpression::new(
                Self::Type::static_type(),
                Some(&self_expression),
                "tag",
            );
            let is_checked_expression = gtk::ClosureExpression::new(
                clone!(@weak obj => @default-return false, move |args| {
                    let tag: Option<Tag> = args[1].get().unwrap();
                    tag.map_or(false, |tag| obj.other_tag_list().contains(tag))
                }),
                &[tag_expression.upcast()],
            );
            is_checked_expression.bind(&self.check_button.get(), "active", None);

            // FIXME This get activated on first launch which makes it try to append an
            // existing tag
            self.check_button
                .connect_active_notify(clone!(@weak obj => move |check_button| {
                    if let Some(tag) = obj.tag() {
                        let tag_name = tag.name();
                        match check_button.is_active() {
                            true => {
                                if !obj.other_tag_list().append(tag) {
                                    log::warn!("Trying to append an existing tag: {}", tag_name);
                                }
                            }
                            false => {
                                if !obj.other_tag_list().remove(tag) {
                                    log::warn!("Trying to remove a tag that doesn't exist in the list: {}", tag_name);
                                }
                            }
                        }
                    }
                }));
        }
    }

    impl WidgetImpl for Row {}
    impl BinImpl for Row {}
}

glib::wrapper! {
    pub struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl Row {
    pub fn new(other_tag_list: &TagList) -> Self {
        glib::Object::new(&[("other-tag-list", other_tag_list)]).expect("Failed to create Row")
    }

    fn other_tag_list(&self) -> TagList {
        self.property("other-tag-list").unwrap().get().unwrap()
    }

    fn tag(&self) -> Option<Tag> {
        self.property("tag").unwrap().get().unwrap()
    }
}
