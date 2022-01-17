use adw::{prelude::*, subclass::prelude::*};
use anyhow::Context;
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone},
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use std::{fs, path::PathBuf};

use crate::{session::Session, spawn, spawn_blocking, utils};

const MAX_BYTES_FILE_SIZE: u64 = 20_000_000;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(
        resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-file-importer-button.ui"
    )]
    pub struct FileImporterButton {
        pub file_chooser: OnceCell<gtk::FileChooserNative>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileImporterButton {
        const NAME: &'static str = "NwtyContentAttachmentViewFileImporterButton";
        type Type = super::FileImporterButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action(
                "file-importer-button.open-file-chooser",
                None,
                move |obj, _, _| {
                    obj.on_open_file_chooser();
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FileImporterButton {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "new-import",
                    &[gio::File::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for FileImporterButton {}
    impl BinImpl for FileImporterButton {}
}

glib::wrapper! {
    pub struct FileImporterButton(ObjectSubclass<imp::FileImporterButton>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl FileImporterButton {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create FileImporterButton")
    }

    pub fn connect_new_import<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &gio::File) + 'static,
    {
        self.connect_local("new-import", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let file = values[1].get::<gio::File>().unwrap();
            f(&obj, &file);
            None
        })
    }

    /// Make sures that the files are less than `MAX_BYTES_FILE_SIZE`
    fn validify_files(files: &gio::ListModel) -> anyhow::Result<Vec<PathBuf>> {
        let mut valid_files = Vec::new();

        for index in 0..files.n_items() {
            let file = files.item(index).unwrap().downcast::<gio::File>().unwrap();
            let file_path = file.path().unwrap();
            let file_byte_size = fs::metadata(&file_path)
                .map(|metadata| metadata.len())
                .with_context(|| format!("Failed to read file at `{}`", file_path.display()))?;

            // TODO maybe make this less strict or remove this restriction
            // or maybe just warn the user that they are trying to save a large file
            anyhow::ensure!(
                file_byte_size < MAX_BYTES_FILE_SIZE,
                "File `{}` exceeds maximum file size of 20 MB",
                file.basename().unwrap().display()
            );

            valid_files.push(file_path);
        }

        Ok(valid_files)
    }

    async fn import_files(&self, files: Vec<PathBuf>) -> anyhow::Result<()> {
        for source_path in files {
            let notes_dir = Session::default().directory();
            let destination_path =
                utils::generate_unique_path(notes_dir, "OtherFile", source_path.extension());
            let destination_file = gio::File::for_path(&destination_path);

            log::info!(
                "Copying file from `{}` to `{}`",
                source_path.display(),
                destination_path.display()
            );

            spawn_blocking!(
                move || fs::copy(&source_path, &destination_path).with_context(|| format!(
                    "Failed to copy `{}` to `{}`",
                    source_path.display(),
                    destination_path.display()
                ))
            )
            .await?;

            self.emit_by_name::<()>("new-import", &[&destination_file]);
        }

        Ok(())
    }

    fn show_error(&self, text: &str, secondary_text: &str) {
        let error_dialog = gtk::MessageDialog::builder()
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

    fn on_accept_response(&self, files: &gio::ListModel) {
        match Self::validify_files(files) {
            Ok(files) => {
                spawn!(clone!(@weak self as obj => async move {
                    if let Err(err) = obj.import_files(files).await {
                        obj.show_error(&err.to_string(), &gettext("Please try again."));
                        log::error!("Error on importing files: {:?}", err);
                    }
                }));
            }
            Err(err) => {
                self.show_error(&err.to_string(), &gettext("Please try again."));
                log::error!("Error on validifying files: {:?}", err);
            }
        }
    }

    fn init_file_chooser(&self) -> gtk::FileChooserNative {
        // FIXME Should not allow folders, this makes it easy to delete an attachment. Additionally,
        // an attachment should not be able to store a folder

        let chooser = gtk::FileChooserNative::builder()
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
                obj.on_accept_response(&chooser.files());
            }
        }));

        chooser
    }

    fn on_open_file_chooser(&self) {
        let imp = imp::FileImporterButton::from_instance(self);

        let chooser = imp.file_chooser.get_or_init(|| self.init_file_chooser());
        chooser.show();
    }
}
