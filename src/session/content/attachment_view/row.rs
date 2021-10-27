use adw::{prelude::*, subclass::prelude::*};
use gtk::{glib, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use crate::model::Attachment;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-row.ui")]
    pub struct Row {
        #[template_child]
        pub path_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub title_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub created_label: TemplateChild<gtk::Label>,

        pub attachment: RefCell<Option<Attachment>>,
        pub title_binding: RefCell<Option<glib::Binding>>,
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

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "attachment" => self.attachment.borrow().to_value(),
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

    pub fn set_attachment(&self, attachment: Option<Attachment>) {
        let imp = imp::Row::from_instance(self);

        if let Some(binding) = imp.title_binding.take() {
            binding.unbind();
        }

        if let Some(ref attachment) = attachment {
            imp.path_label
                .set_label(attachment.file().path().unwrap().to_str().unwrap());
            imp.created_label
                .set_label(&attachment.created().fuzzy_display());

            let binding = attachment
                .bind_property("title", &imp.title_entry.get(), "text")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();
            imp.title_binding.replace(binding);
        } else {
            imp.path_label.set_label("This row has no attachment");
            imp.created_label.set_label("This row has no attachment");
        }

        imp.attachment.replace(attachment);
        self.notify("attachment");
    }
}
