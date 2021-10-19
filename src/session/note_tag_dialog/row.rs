use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::RefCell;

use super::{NoteTagLists, Tag};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/note-tag-dialog-row.ui")]
    pub struct Row {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub check_button: TemplateChild<gtk::CheckButton>,

        pub other_tag_lists: RefCell<NoteTagLists>,
        pub tag: RefCell<Option<Tag>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "NwtyNoteTagDialogRow";
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
                    glib::ParamSpec::new_boxed(
                        "other-tag-lists",
                        "A list of other tag lists",
                        "The tag list to compare with",
                        NoteTagLists::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "tag",
                        "tag",
                        "The tag represented by this row",
                        Tag::static_type(),
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
                "other-tag-lists" => {
                    let other_tag_lists = value.get().unwrap();
                    self.other_tag_lists.replace(other_tag_lists);
                }
                "tag" => {
                    let tag = value.get().unwrap();
                    obj.set_tag(tag);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "other-tag-lists" => self.other_tag_lists.borrow().to_value(),
                "tag" => obj.tag().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_signals();
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
    pub fn new(other_tag_lists: &NoteTagLists) -> Self {
        glib::Object::new(&[("other-tag-lists", other_tag_lists)]).expect("Failed to create Row")
    }

    pub fn tag(&self) -> Option<Tag> {
        let imp = imp::Row::from_instance(self);
        imp.tag.borrow().clone()
    }

    pub fn set_tag(&self, tag: Option<Tag>) {
        let imp = imp::Row::from_instance(self);

        if let Some(ref tag) = tag {
            let other_tag_lists = self.other_tag_lists();

            let mut is_all_contain_tag = true;
            let mut is_all_not_contain_tag = true;

            for tag_list in other_tag_lists.iter() {
                if !is_all_contain_tag && !is_all_not_contain_tag {
                    break;
                }

                if tag_list.contains(tag) {
                    is_all_not_contain_tag = false;
                } else {
                    is_all_contain_tag = false;
                }
            }

            // Basically impossible to get empty other_tag_lists from the ui, but just to be sure.
            if other_tag_lists.is_empty() {
                log::error!("Other tag lists found to be empty");
                is_all_contain_tag = false;
                is_all_not_contain_tag = true;
            }

            if is_all_contain_tag {
                imp.check_button.set_active(true);
            } else if is_all_not_contain_tag {
                imp.check_button.set_active(false);
            } else {
                // Some tag list contain but not all or vice versa
                imp.check_button.set_inconsistent(true);
            }
        }

        imp.tag.replace(tag);
        self.notify("tag");
    }

    fn other_tag_lists(&self) -> NoteTagLists {
        self.property("other-tag-lists").unwrap().get().unwrap()
    }

    fn setup_signals(&self) {
        let imp = imp::Row::from_instance(self);

        // FIXME This get activated on first launch which makes it try to append an
        // existing tag
        imp.check_button
            .connect_active_notify(clone!(@weak self as obj => move |check_button| {
                let tag = match obj.tag() {
                    Some(tag) => tag,
                    None => return,
                };

                let imp = imp::Row::from_instance(&obj);
                imp.check_button.set_inconsistent(false);

                if check_button.is_active() {
                    for tag_list in obj.other_tag_lists().iter() {
                        if tag_list.append(tag.clone()).is_err() {
                            log::warn!("Trying to append an existing tag: {}", tag.name());
                        }
                    }
                } else {
                    for tag_list in obj.other_tag_lists().iter() {
                        if tag_list.remove(&tag).is_err() {
                            log::warn!(
                                "Trying to remove a tag that doesn't exist in the list: {}",
                                tag.name()
                            );
                        }
                    }
                }
            }));

        // TODO Implement this so clicking the row activates the checkbutton
        // Works well when clicking the row but when you click the button it gets activated
        // twice, Idk how to not let the click pass through both widgets
        // let gesture = gtk::GestureClick::new();
        // gesture.connect_pressed(clone!(@weak self as obj => move |_,_,_,_| {
        //     let imp = imp::Row::from_instance(&obj);
        //     imp.check_button.activate();
        // }));
        // self.add_controller(&gesture);
    }
}
