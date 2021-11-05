mod visualizer;

use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::{sync::Lazy, unsync::OnceCell};

use std::cell::RefCell;

use self::visualizer::Visualizer;
use crate::{core::AudioRecording, spawn, utils};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(
        resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-audio-recorder-button.ui"
    )]
    pub struct AudioRecorderButton {
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub visualizer: TemplateChild<Visualizer>,

        pub recording: RefCell<Option<AudioRecording>>,
        pub record_done_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub peak_notify_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub popover_closed_handler_id: OnceCell<glib::SignalHandlerId>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioRecorderButton {
        const NAME: &'static str = "NwtyContentAttachmentViewAudioRecorderButton";
        type Type = super::AudioRecorderButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Visualizer::static_type();
            Self::bind_template(klass);

            klass.install_action("audio-recorder-button.record-ok", None, move |obj, _, _| {
                obj.stop_recording();

                let imp = AudioRecorderButton::from_instance(obj);
                let popover_closed_handler_id = imp.popover_closed_handler_id.get().unwrap();
                imp.popover.block_signal(popover_closed_handler_id);
                imp.menu_button.popdown();
                imp.popover.unblock_signal(popover_closed_handler_id);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AudioRecorderButton {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("on-record", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_signals();
        }
    }

    impl WidgetImpl for AudioRecorderButton {}
    impl BinImpl for AudioRecorderButton {}
}

glib::wrapper! {
    pub struct AudioRecorderButton(ObjectSubclass<imp::AudioRecorderButton>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl AudioRecorderButton {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AudioRecorderButton")
    }

    pub fn connect_on_record<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_local("on-record", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
        .unwrap()
    }

    fn recording(&self) -> AudioRecording {
        let imp = imp::AudioRecorderButton::from_instance(self);
        imp.recording
            .borrow()
            .clone()
            .expect("Recording not setup or already disposed")
    }

    fn start_recording(&self) {
        let file = {
            let mut file_path = utils::default_notes_dir();
            file_path.push(utils::generate_unique_file_name("AudioRecording"));
            file_path.set_extension("ogg");
            gio::File::for_path(&file_path)
        };
        let recording = AudioRecording::new(&file);

        if let Err(err) = recording.start() {
            log::error!("Failed to start recording: {}", err);
            return;
        }

        self.emit_by_name("on-record", &[]).unwrap();

        let imp = imp::AudioRecorderButton::from_instance(self);

        let record_done_handler_id =
            recording.connect_record_done(clone!(@weak self as obj => move |_, res| {
                obj.dispose_recording();

                let imp = imp::AudioRecorderButton::from_instance(&obj);
                imp.visualizer.clear_peaks();

                // TODO append successful recording to attachments

                log::error!("{:?}", res);
            }));
        imp.record_done_handler_id
            .replace(Some(record_done_handler_id));

        let peak_notify_handler_id =
            recording.connect_peak_notify(clone!(@weak self as obj => move |recording,_| {
                let peak = 10_f64.powf(recording.peak() / 20.0);

                let imp = imp::AudioRecorderButton::from_instance(&obj);
                imp.visualizer.push_peak(peak);
            }));
        imp.peak_notify_handler_id
            .replace(Some(peak_notify_handler_id));

        imp.recording.replace(Some(recording));

        log::info!("Started recording");
    }

    fn cancel_recording(&self) {
        let recording = self.recording();

        recording.stop();

        spawn!(async move {
            if let Err(err) = recording.delete().await {
                log::warn!("Failed to delete recording: {}", err);
            }
        });

        log::info!("Cancelled recording");
    }

    fn stop_recording(&self) {
        self.recording().stop();

        log::info!("Stopped recording");
    }

    fn dispose_recording(&self) {
        let imp = imp::AudioRecorderButton::from_instance(self);

        let recording = imp.recording.take().unwrap();
        recording.disconnect(imp.record_done_handler_id.take().unwrap());
        recording.disconnect(imp.peak_notify_handler_id.take().unwrap());
    }

    fn setup_signals(&self) {
        let imp = imp::AudioRecorderButton::from_instance(self);

        imp.popover
            .connect_show(clone!(@weak self as obj => move |_| {
                obj.start_recording();
            }));

        let popover_closed_handler_id =
            imp.popover
                .connect_closed(clone!(@weak self as obj => move |_| {
                    obj.cancel_recording();
                }));
        imp.popover_closed_handler_id
            .set(popover_closed_handler_id)
            .unwrap();
    }
}
