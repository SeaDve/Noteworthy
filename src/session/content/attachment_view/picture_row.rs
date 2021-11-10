use adw::{prelude::*, subclass::prelude::*};
use gtk::{glib, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use crate::{model::Attachment, widgets::RoundedPicture};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-picture-row.ui")]
    pub struct PictureRow {
        #[template_child]
        pub picture: TemplateChild<RoundedPicture>,

        pub attachment: RefCell<Attachment>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PictureRow {
        const NAME: &'static str = "NwtyContentAttachmentViewPictureRow";
        type Type = super::PictureRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            RoundedPicture::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PictureRow {
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

    impl WidgetImpl for PictureRow {}
    impl BinImpl for PictureRow {}
}

glib::wrapper! {
    pub struct PictureRow(ObjectSubclass<imp::PictureRow>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl PictureRow {
    pub fn new(attachment: &Attachment) -> Self {
        glib::Object::new(&[("attachment", attachment)]).expect("Failed to create PictureRow")
    }

    fn set_attachment(&self, attachment: Attachment) {
        let imp = imp::PictureRow::from_instance(self);

        imp.picture.set_file(Some(&attachment.file()));

        imp.attachment.replace(attachment);
        self.notify("attachment");
    }

    fn attachment(&self) -> Attachment {
        let imp = imp::PictureRow::from_instance(self);
        imp.attachment.borrow().clone()
    }
}