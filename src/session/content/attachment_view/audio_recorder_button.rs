use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gio,
    glib::{self, clone},
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use crate::{
    core::AudioRecorder,
    session::Session,
    spawn,
    widgets::{AudioVisualizer, TimeLabel},
};

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

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
        pub visualizer: TemplateChild<AudioVisualizer>,
        #[template_child]
        pub duration_label: TemplateChild<TimeLabel>,

        pub recorder: AudioRecorder,
        pub popover_closed_handler_id: OnceCell<glib::SignalHandlerId>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioRecorderButton {
        const NAME: &'static str = "NwtyContentAttachmentViewAudioRecorderButton";
        type Type = super::AudioRecorderButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("audio-recorder-button.record-ok", None, move |obj, _, _| {
                obj.stop_recording();

                let imp = obj.imp();
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
                vec![
                    Signal::builder("on-record", &[], <()>::static_type().into()).build(),
                    Signal::builder(
                        "record-done",
                        &[gio::File::static_type().into()],
                        <()>::static_type().into(),
                    )
                    .build(),
                ]
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
    }

    pub fn connect_record_done<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &gio::File) + 'static,
    {
        self.connect_local("record-done", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let file = values[1].get::<gio::File>().unwrap();
            f(&obj, &file);
            None
        })
    }

    fn visualizer(&self) -> &AudioVisualizer {
        &self.imp().visualizer
    }

    fn duration_label(&self) -> &TimeLabel {
        &self.imp().duration_label
    }

    fn recorder(&self) -> &AudioRecorder {
        &self.imp().recorder
    }

    fn start_recording(&self) {
        let recording_base_path = Session::default().directory();

        if let Err(err) = self.recorder().start(&recording_base_path) {
            log::error!("Failed to start recording: {:?}", err);
            return;
        }

        self.emit_by_name::<()>("on-record", &[]);

        log::info!("Started recording");
    }

    fn cancel_recording(&self) {
        spawn!(clone!(@weak self as obj => async move {
            obj.recorder().cancel().await;
        }));

        self.visualizer().clear_peaks();
        self.duration_label().reset();

        log::info!("Cancelled recording");
    }

    fn stop_recording(&self) {
        spawn!(clone!(@weak self as obj => async move {
            match obj.recorder().stop().await {
                Ok(recording) => {
                    obj.emit_by_name::<()>("record-done", &[&recording.into_file()]);
                }
                Err(err) => {
                    log::error!("Failed to stop recording: {:?}", err);
                }
            }
        }));

        self.visualizer().clear_peaks();
        self.duration_label().reset();

        log::info!("Stopped recording");
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        imp.recorder
            .connect_peak_notify(clone!(@weak self as obj => move |recorder| {
                let peak = 10_f64.powf(recorder.peak() / 20.0);
                obj.visualizer().push_peak(peak as f32);
            }));

        imp.recorder
            .connect_duration_notify(clone!(@weak self as obj => move |recorder| {
                obj.duration_label().set_time(recorder.duration());
            }));

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
