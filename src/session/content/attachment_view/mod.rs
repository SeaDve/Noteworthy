mod audio_recorder_button;
mod audio_row;
mod camera_button;
mod file_importer_button;
mod other_row;
mod picture_row;
mod row;

use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
    CompositeTemplate,
};

use self::{
    audio_recorder_button::AudioRecorderButton, audio_row::AudioRow, camera_button::CameraButton,
    file_importer_button::FileImporterButton, other_row::OtherRow, picture_row::PictureRow,
    row::Row,
};
use crate::{
    core::AudioPlayerHandler,
    model::{Attachment, AttachmentList, DateTime},
    spawn,
    utils::PropExpr,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view.ui")]
    pub struct AttachmentView {
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub selection: TemplateChild<gtk::NoSelection>,
        #[template_child]
        pub audio_recorder_button: TemplateChild<AudioRecorderButton>,
        #[template_child]
        pub camera_button: TemplateChild<CameraButton>,
        #[template_child]
        pub file_importer_button: TemplateChild<FileImporterButton>,

        pub audio_player_handler: AudioPlayerHandler,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttachmentView {
        const NAME: &'static str = "NwtyContentAttachmentView";
        type Type = super::AttachmentView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            AudioRecorderButton::static_type();
            FileImporterButton::static_type();
            CameraButton::static_type();
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

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_list_view();
            obj.setup_signals();
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
        self.notify("attachment-list");
    }

    fn attachment_list(&self) -> Option<AttachmentList> {
        let imp = imp::AttachmentView::from_instance(self);
        imp.selection.model().map(|model| model.downcast().unwrap())
    }

    fn setup_list_view(&self) {
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(clone!(@weak self as obj => move |_, list_item| {
            let attachment_row = Row::new();

            attachment_row.connect_on_delete(move |attachment_row| {
                let attachment = attachment_row.attachment().unwrap();

                let attachment_list = obj
                    .attachment_list()
                    .expect("No current attachment list on attachment view");

                attachment_list.remove(&attachment).unwrap();

                spawn!(async move {
                    attachment.delete().await;
                });
            });

            list_item
                .property_expression("item")
                .bind(&attachment_row, "attachment", None::<&gtk::Widget>);

            list_item.set_child(Some(&attachment_row));
            list_item.set_activatable(false);
        }));

        factory.connect_bind(clone!(@weak self as obj => move |_, list_item| {
            let attachment_row: Row = list_item.child().unwrap().downcast().unwrap();

            if let Some(ref audio_row) = attachment_row.inner_row::<AudioRow>() {
                let imp = imp::AttachmentView::from_instance(&obj);
                imp.audio_player_handler.append(audio_row.audio_player().clone());
            }
        }));

        factory.connect_unbind(clone!(@weak self as obj => move |_, list_item| {
            let attachment_row: Row = list_item.child().unwrap().downcast().unwrap();

            if let Some(ref audio_row) = attachment_row.inner_row::<AudioRow>() {
                let imp = imp::AttachmentView::from_instance(&obj);
                imp.audio_player_handler.remove(audio_row.audio_player());
            }
        }));

        let imp = imp::AttachmentView::from_instance(self);
        imp.list_view.set_factory(Some(&factory));
    }

    fn setup_signals(&self) {
        let imp = imp::AttachmentView::from_instance(self);

        imp.audio_recorder_button
            .connect_on_record(clone!(@weak self as obj => move |_| {
                let imp = imp::AttachmentView::from_instance(&obj);
                imp.audio_player_handler.stop_all();
            }));

        imp.audio_recorder_button
            .connect_record_done(clone!(@weak self as obj => move |_, file| {
                let attachment_list = obj
                    .attachment_list()
                    .expect("No current attachment list on attachment view");

                let new_attachment = Attachment::new(file, &DateTime::now());
                attachment_list.append(new_attachment).unwrap();
            }));

        imp.camera_button
            .connect_capture_done(clone!(@weak self as obj => move |_, file| {
                let attachment_list = obj
                    .attachment_list()
                    .expect("No current attachment list on attachment view");

                let new_attachment = Attachment::new(file, &DateTime::now());
                attachment_list.append(new_attachment).unwrap();
            }));

        imp.file_importer_button
            .connect_new_import(clone!(@weak self as obj => move |_, file| {
                let attachment_list = obj
                    .attachment_list()
                    .expect("No current attachment list on attachment view");

                let new_attachment = Attachment::new(file, &DateTime::now());
                attachment_list.append(new_attachment).unwrap();
            }));
    }
}
