use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::{sync::Lazy, unsync::OnceCell};

use std::{fs, path::PathBuf};

use crate::{spawn, spawn_blocking, utils};

const MAX_BYTES_FILE_SIZE: u64 = 20_000_000;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(
        resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-file-opener-button.ui"
    )]
    pub struct FileOpenerButton {
        #[template_child]
        pub button: TemplateChild<gtk::Button>,

        pub file_chooser: OnceCell<gtk::FileChooserNative>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileOpenerButton {
        const NAME: &'static str = "NwtyContentAttachmentViewFileOpenerButton";
        type Type = super::FileOpenerButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("file-opener-button.open-file", None, move |obj, _, _| {
                obj.on_open_file();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FileOpenerButton {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "open-done",
                    &[gio::File::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // obj.setup_signals();
        }
    }

    impl WidgetImpl for FileOpenerButton {}
    impl BinImpl for FileOpenerButton {}
}

glib::wrapper! {
    pub struct FileOpenerButton(ObjectSubclass<imp::FileOpenerButton>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl FileOpenerButton {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create FileOpenerButton")
    }

    pub fn connect_open_done<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &gio::File) + 'static,
    {
        self.connect_local("open-done", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let file = values[1].get::<gio::File>().unwrap();
            f(&obj, &file);
            None
        })
        .unwrap()
    }

    fn init_file_chooser(&self) -> gtk::FileChooserNative {
        let chooser = gtk::FileChooserNativeBuilder::new()
            .accept_label(&gettext("Select"))
            .cancel_label(&gettext("Cancel"))
            .title(&gettext("Select Files to Import"))
            .action(gtk::FileChooserAction::Open)
            .select_multiple(true)
            .modal(true)
            .build();

        chooser.set_transient_for(
            self.root()
                .map(|w| w.downcast::<gtk::Window>().unwrap())
                .as_ref(),
        );

        chooser.connect_response(clone!(@weak self as obj => move |chooser, response| {
            if response == gtk::ResponseType::Accept {
                obj.on_accept_response(chooser.files().unwrap());
            }
        }));

        chooser
    }

    fn on_accept_response(&self, files: gio::ListModel) {
        let mut files_to_import = Vec::new();

        for index in 0..files.n_items() {
            let file = files.item(index).unwrap().downcast::<gio::File>().unwrap();
            let file_path = file.path().unwrap();
            let file_byte_size = fs::metadata(&file_path).map(|metadata| metadata.len());

            match file_byte_size {
                Ok(byte_size) => {
                    // TODO maybe increase or remove this restriction?
                    if byte_size >= MAX_BYTES_FILE_SIZE {
                        self.show_error(
                            &gettext!(
                                "File {} exceeds maximum file size of 20 MB",
                                file.basename().unwrap().display(),
                            ),
                            &gettext("Please try other file."),
                        );
                        log::info!(
                            "File at {} exceeds max size of {} B",
                            file_path.display(),
                            byte_size
                        );
                        return;
                    }

                    files_to_import.push(file_path);
                }
                Err(err) => {
                    self.show_error(
                        &gettext("An error occurred while getting file info"),
                        &gettext("Please try again."),
                    );
                    log::error!("Failed to query file info: {:#}", err);
                }
            }
        }

        spawn!(clone!(@weak self as obj => async move {
            obj.import_files(files_to_import).await;
        }));
    }

    fn on_open_file(&self) {
        let imp = imp::FileOpenerButton::from_instance(self);

        let chooser = imp.file_chooser.get_or_init(|| self.init_file_chooser());
        chooser.show();
    }

    fn show_error(&self, text: &str, secondary_text: &str) {
        let error_dialog = gtk::MessageDialogBuilder::new()
            .text(text)
            .secondary_text(secondary_text)
            .buttons(gtk::ButtonsType::Ok)
            .message_type(gtk::MessageType::Error)
            .modal(true)
            .build();

        error_dialog.set_transient_for(
            self.root()
                .map(|w| w.downcast::<gtk::Window>().unwrap())
                .as_ref(),
        );

        error_dialog.connect_response(|error_dialog, _| error_dialog.destroy());
        error_dialog.present();
    }

    async fn import_files(&self, files: Vec<PathBuf>) {
        let default_notes_dir = utils::default_notes_dir();

        for source_path in files {
            let destination_path = {
                let file_name = utils::generate_unique_file_name("OtherFile");
                let mut destination_path = default_notes_dir.join(file_name);

                if let Some(extension) = source_path.extension() {
                    destination_path.set_extension(extension);
                }

                destination_path
            };

            let destination_file = gio::File::for_path(&destination_path);

            log::info!(
                "Copying file from {} to {}",
                source_path.display(),
                destination_path.display()
            );

            match spawn_blocking!(move || fs::copy(&source_path, &destination_path)).await {
                Ok(_) => {
                    self.emit_by_name("open-done", &[&destination_file])
                        .unwrap();
                }
                Err(err) => {
                    log::error!("An error occurred while copying file: {}", err);
                }
            }
        }
    }
}
