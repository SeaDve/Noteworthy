// TODO remove this file, since it is not currently used

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/file-chooser-button.ui")]
    pub struct FileChooserButton {
        #[template_child]
        pub inner_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        pub current_folder: RefCell<Option<gio::File>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileChooserButton {
        const NAME: &'static str = "NwtyFileChooserButton";
        type Type = super::FileChooserButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("file-chooser-button.choose", None, move |obj, _, _| {
                let chooser = gtk::FileChooserDialogBuilder::new()
                    .transient_for(&obj.root().unwrap().downcast::<gtk::Window>().unwrap())
                    .modal(true)
                    .action(gtk::FileChooserAction::SelectFolder)
                    .title(&gettext("Select a Folder"))
                    .build();

                chooser.add_button(&gettext("_Cancel"), gtk::ResponseType::Cancel);
                chooser.add_button(&gettext("_Select"), gtk::ResponseType::Accept);
                chooser.set_default_response(gtk::ResponseType::Accept);

                if let Some(current_folder) = obj.current_folder() {
                    chooser.set_current_folder(&current_folder).unwrap();
                }

                chooser.run_async(clone!(@weak obj => move |chooser,response| {
                    if response == gtk::ResponseType::Accept {
                        obj.set_current_folder(chooser.file());
                    }
                    chooser.close();
                }));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FileChooserButton {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "current-folder",
                    "Current Folder",
                    "Current folder in self",
                    gio::File::static_type(),
                    glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "current-folder" => {
                    let current_folder = value.get().unwrap();
                    obj.set_current_folder(current_folder);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "current-folder" => obj.current_folder().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for FileChooserButton {}
    impl BinImpl for FileChooserButton {}
}

glib::wrapper! {
    pub struct FileChooserButton(ObjectSubclass<imp::FileChooserButton>)
        @extends gtk::Widget, adw::Bin, @implements gtk::Accessible;
}

impl FileChooserButton {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create FileChooserButton.")
    }

    pub fn set_current_folder(&self, current_folder: Option<gio::File>) {
        let imp = imp::FileChooserButton::from_instance(self);

        if let Some(ref current_folder) = current_folder {
            imp.label.set_label(&current_folder.parse_name());
        } else {
            imp.label.set_label("");
        }

        imp.current_folder.replace(current_folder);
        self.notify("current-folder");
    }

    pub fn current_folder(&self) -> Option<gio::File> {
        let imp = imp::FileChooserButton::from_instance(self);
        imp.current_folder.borrow().clone()
    }
}
