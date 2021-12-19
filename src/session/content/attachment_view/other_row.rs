use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use std::cell::RefCell;

use crate::model::Attachment;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-other-row.ui")]
    pub struct OtherRow {
        pub attachment: RefCell<Attachment>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OtherRow {
        const NAME: &'static str = "NwtyContentAttachmentViewOtherRow";
        type Type = super::OtherRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("other-row.launch-file", None, move |obj, _, _| {
                obj.on_launch_file();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for OtherRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "attachment",
                    "attachment",
                    "The attachment represented by this row",
                    Attachment::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
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
                "attachment" => {
                    let attachment = value.get().unwrap();
                    self.attachment.replace(attachment);
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

            obj.setup_gesture();
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for OtherRow {}
}

glib::wrapper! {
    pub struct OtherRow(ObjectSubclass<imp::OtherRow>)
        @extends gtk::Widget;
}

impl OtherRow {
    pub fn new(attachment: &Attachment) -> Self {
        glib::Object::new(&[("attachment", attachment)]).expect("Failed to create OtherRow")
    }

    fn attachment(&self) -> Attachment {
        let imp = imp::OtherRow::from_instance(self);
        imp.attachment.borrow().clone()
    }

    fn on_launch_file(&self) {
        let file_uri = self.attachment().file().uri();
        let res = gio::AppInfo::launch_default_for_uri(&file_uri, None::<&gio::AppLaunchContext>);

        if let Err(err) = res {
            log::error!("Failed to open file at uri `{}`: {}", file_uri, err);
            // TODO show user facing error
        }
    }

    fn setup_gesture(&self) {
        let gesture = gtk::GestureClick::new();
        gesture.connect_released(clone!(@weak self as obj => move |_, _, _, _| {
            obj.activate_action("other-row.launch-file", None);
        }));
        self.add_controller(&gesture);
    }
}
