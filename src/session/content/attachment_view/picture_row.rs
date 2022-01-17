use gtk::{
    gdk, gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use std::cell::RefCell;

use crate::{model::Attachment, spawn, spawn_blocking};

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-picture-row.ui")]
    pub struct PictureRow {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,

        pub attachment: RefCell<Attachment>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PictureRow {
        const NAME: &'static str = "NwtyContentAttachmentViewPictureRow";
        type Type = super::PictureRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PictureRow {
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

    impl WidgetImpl for PictureRow {}
}

glib::wrapper! {
    pub struct PictureRow(ObjectSubclass<imp::PictureRow>)
        @extends gtk::Widget;
}

impl PictureRow {
    pub fn new(attachment: &Attachment) -> Self {
        glib::Object::new(&[("attachment", attachment)]).expect("Failed to create PictureRow")
    }

    fn set_attachment(&self, attachment: Attachment) {
        if attachment == self.attachment() {
            return;
        }

        let file = attachment.file();
        let path = file.path().unwrap();

        spawn!(clone!(@weak self as obj => async move {
            match obj.load_texture_from_file(file).await {
                Ok(ref texture) => {
                    obj.imp().picture.set_paintable(Some(texture));
                }
                Err(err) => {
                    log::error!(
                        "Failed to load texture from file `{}`: {:?}",
                        path.display(),
                        err
                    );
                }
            }
        }));

        self.imp().attachment.replace(attachment);
        self.notify("attachment");
    }

    fn attachment(&self) -> Attachment {
        self.imp().attachment.borrow().clone()
    }

    async fn load_texture_from_file(&self, file: gio::File) -> Result<gdk::Texture, glib::Error> {
        spawn_blocking!(move || gdk::Texture::from_file(&file)).await
    }
}
