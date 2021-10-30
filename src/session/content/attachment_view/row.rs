use adw::{prelude::*, subclass::prelude::*};
use gtk::{glib, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use super::{AudioRow, OtherRow};
use crate::model::{Attachment, AttachmentKind};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-row.ui")]
    pub struct Row {
        #[template_child]
        pub content: TemplateChild<adw::Bin>,

        pub attachment: RefCell<Option<Attachment>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "NwtyContentAttachmentViewRow";
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
                vec![glib::ParamSpec::new_object(
                    "attachment",
                    "attachment",
                    "The attachment represented by this row",
                    Attachment::static_type(),
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
                "attachment" => {
                    let attachment = value.get().unwrap();
                    obj.set_attachment(attachment);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "attachment" => obj.attachment().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
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
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Row")
    }

    pub fn attachment(&self) -> Option<Attachment> {
        let imp = imp::Row::from_instance(self);
        imp.attachment.borrow().clone()
    }

    pub fn set_attachment(&self, attachment: Option<Attachment>) {
        if attachment == self.attachment() {
            return;
        }

        let imp = imp::Row::from_instance(self);

        if let Some(ref attachment) = attachment {
            self.replace_child(attachment);
        } else {
            self.remove_child();
        }

        imp.attachment.replace(attachment);
        self.notify("attachment");
    }

    pub fn inner_row<T: IsA<gtk::Widget>>(&self) -> Option<T> {
        let imp = imp::Row::from_instance(self);
        imp.content.child().and_then(|w| w.downcast::<T>().ok())
    }

    fn replace_child(&self, attachment: &Attachment) {
        let child: gtk::Widget = match attachment.kind() {
            AttachmentKind::Ogg => AudioRow::new(attachment).upcast(),
            AttachmentKind::Other => OtherRow::new(attachment).upcast(),
        };

        let imp = imp::Row::from_instance(self);
        imp.content.set_child(Some(&child));
    }

    fn remove_child(&self) {
        let imp = imp::Row::from_instance(self);
        imp.content.set_child(None::<&gtk::Widget>);
    }
}
