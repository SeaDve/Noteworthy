mod audio_row;
mod other_row;
mod row;

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use self::{audio_row::AudioRow, other_row::OtherRow, row::Row};
use crate::model::AttachmentList;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view.ui")]
    pub struct AttachmentView {
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub selection: TemplateChild<gtk::NoSelection>,

        pub attachment_list: RefCell<Option<AttachmentList>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttachmentView {
        const NAME: &'static str = "NwtyContentAttachmentView";
        type Type = super::AttachmentView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Row::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AttachmentView {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "attachment-list",
                    "Attachment List",
                    "List containing the attachments",
                    AttachmentList::static_type(),
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
                "attachment-list" => {
                    let attachment_list = value.get().unwrap();
                    obj.set_attachment_list(attachment_list);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "attachment-list" => self.attachment_list.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for AttachmentView {}
    impl BinImpl for AttachmentView {}
}

glib::wrapper! {
    pub struct AttachmentView(ObjectSubclass<imp::AttachmentView>)
        @extends gtk::Widget, adw::Bin;
}

impl AttachmentView {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AttachmentView.")
    }

    pub fn set_attachment_list(&self, attachment_list: Option<AttachmentList>) {
        let imp = imp::AttachmentView::from_instance(self);

        imp.selection.set_model(attachment_list.as_ref());

        imp.attachment_list.replace(attachment_list);
        self.notify("attachment-list");
    }
}
