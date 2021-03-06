use adw::prelude::*;
use gtk::{glib, subclass::prelude::*};

use std::cell::RefCell;

use super::{AudioRow, OtherRow, PictureRow};
use crate::{core::FileType, model::Attachment};

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

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
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("row.delete-attachment", None, move |obj, _, _| {
                obj.emit_by_name::<()>("on-delete", &[]);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Row {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("on-delete", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
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

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Row {}
}

glib::wrapper! {
    pub struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget;
}

impl Row {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Row")
    }

    pub fn attachment(&self) -> Option<Attachment> {
        self.imp().attachment.borrow().clone()
    }

    pub fn set_attachment(&self, attachment: Option<Attachment>) {
        if attachment == self.attachment() {
            return;
        }

        if let Some(ref attachment) = attachment {
            self.replace_child(attachment);
        } else {
            self.remove_child();
        }

        self.imp().attachment.replace(attachment);
        self.notify("attachment");
    }

    pub fn inner_row<T: IsA<gtk::Widget>>(&self) -> Option<T> {
        self.imp()
            .content
            .child()
            .and_then(|w| w.downcast::<T>().ok())
    }

    pub fn connect_on_delete<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_local("on-delete", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
    }

    fn replace_child(&self, attachment: &Attachment) {
        // TODO make other row activatable too
        let child: gtk::Widget = match attachment.file_type() {
            FileType::Audio => {
                self.remove_css_class("activatable");
                AudioRow::new(attachment).upcast()
            }
            FileType::Bitmap => {
                self.add_css_class("activatable");
                PictureRow::new(attachment).upcast()
            }
            FileType::Markdown | FileType::Unknown => {
                self.add_css_class("activatable");
                OtherRow::new(attachment).upcast()
            }
        };

        self.imp().content.set_child(Some(&child));
    }

    fn remove_child(&self) {
        self.imp().content.set_child(gtk::Widget::NONE);
    }
}
