use gst_pbutils::prelude::*;
use gtk::{
    gio,
    glib::{self, clone, subclass::Signal, GBoxed},
    subclass::prelude::*,
};
use once_cell::{sync::Lazy, unsync::OnceCell};

#[derive(Debug, Clone, GBoxed)]
#[gboxed(type_name = "NwtyAudioRecordingResult")]
pub enum AudioRecordingResult {
    Ok(gio::File),
    Err(String),
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AudioRecording {
        pub file: OnceCell<gio::File>,
        pub pipeline: OnceCell<gst::Pipeline>,
        pub bus: OnceCell<gst::Bus>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioRecording {
        const NAME: &'static str = "NwtyAudioRecording";
        type Type = super::AudioRecording;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AudioRecording {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "record-done",
                    &[AudioRecordingResult::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "file",
                    "file",
                    "File where the recording is saved",
                    gio::File::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "file" => {
                    let file = value.get().unwrap();
                    self.file.set(file).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.file.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_pipeline();
        }
    }
}

glib::wrapper! {
    pub struct AudioRecording(ObjectSubclass<imp::AudioRecording>);
}

impl AudioRecording {
    pub fn new(file: &gio::File) -> Self {
        glib::Object::new::<Self>(&[("file", file)]).expect("Failed to create AudioRecording.")
    }

    pub fn connect_record_done<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, AudioRecordingResult) + 'static,
    {
        self.connect_local("record-done", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let result = values[1].get::<AudioRecordingResult>().unwrap();
            f(&obj, result);
            None
        })
        .unwrap()
    }

    pub fn start(&self) {
        let pipeline = self.pipeline();

        let bus = pipeline.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak self as obj => @default-return glib::Continue(false), move |_,message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        let imp = imp::AudioRecording::from_instance(self);
        imp.bus.set(bus).unwrap();

        pipeline.set_state(gst::State::Playing).unwrap();
    }

    pub fn pause(&self) {
        self.pipeline().set_state(gst::State::Paused).unwrap();
    }

    pub fn resume(&self) {
        self.pipeline().set_state(gst::State::Playing).unwrap();
    }

    pub fn stop(&self) {
        log::info!("Sending EOS event to pipeline");
        self.pipeline().send_event(gst::event::Eos::new());
    }

    fn file(&self) -> gio::File {
        self.property("file").unwrap().get().unwrap()
    }

    fn pipeline(&self) -> gst::Pipeline {
        let imp = imp::AudioRecording::from_instance(self);
        imp.pipeline.get().expect("Pipeline not setup").clone()
    }

    fn default_audio_source_name(&self) -> String {
        let res = pulsectl::controllers::SourceController::create()
            .and_then(|mut controller| controller.get_server_info())
            .and_then(|server_info| {
                server_info.default_source_name.ok_or_else(|| {
                    pulsectl::ControllerError::GetInfo("default source name not found".into())
                })
            });

        match res {
            Ok(audio_source_name) => audio_source_name,
            Err(err) => {
                log::warn!("Failed to get audio source name: {}", err);
                log::warn!("Falling back to default");
                String::new()
            }
        }
    }

    fn encodebin_profile(&self) -> gst_pbutils::EncodingContainerProfile {
        let encoding_profile = gst_pbutils::EncodingAudioProfileBuilder::new()
            .format(&gst::Caps::builder("audio/x-opus").build())
            .presence(1)
            .build()
            .unwrap();

        gst_pbutils::EncodingContainerProfileBuilder::new()
            .format(&gst::Caps::builder("application/ogg").build())
            .add_profile(&encoding_profile)
            .build()
            .unwrap()
    }

    fn emit_record_done(&self, result: AudioRecordingResult) {
        self.emit_by_name("record-done", &[&result]).unwrap();
    }

    fn setup_pipeline(&self) {
        let pipeline = gst::Pipeline::new(None);

        let src = gst::ElementFactory::make("pulsesrc", None).unwrap();
        src.set_property("device", &self.default_audio_source_name())
            .unwrap();

        let audioconvert = gst::ElementFactory::make("audioconvert", None).unwrap();
        let level = gst::ElementFactory::make("level", None).unwrap();

        let encodebin = gst::ElementFactory::make("encodebin", None).unwrap();
        encodebin
            .set_property("profile", &self.encodebin_profile())
            .unwrap();

        let filesink = gst::ElementFactory::make("filesink", None).unwrap();
        filesink.set_property("location", &self.file()).unwrap();

        pipeline
            .add_many(&[&src, &audioconvert, &level, &encodebin, &filesink])
            .unwrap();

        src.link(&audioconvert).unwrap();
        audioconvert
            .link_filtered(&level, &gst::Caps::builder("audio/x-raw").build())
            .unwrap();
        level.link(&encodebin).unwrap();
        encodebin.link(&filesink).unwrap();

        let imp = imp::AudioRecording::from_instance(self);
        imp.pipeline.set(pipeline).unwrap();
    }

    fn dispose_pipeline(&self) {
        let imp = imp::AudioRecording::from_instance(self);
        let bus = imp.bus.get().unwrap();
        bus.remove_watch().unwrap();

        self.pipeline().set_state(gst::State::Null).unwrap();
    }

    fn handle_bus_message(&self, message: &gst::Message) -> glib::Continue {
        match message.view() {
            gst::MessageView::Eos(_) => {
                log::info!("Eos signal received from record bus");
                self.dispose_pipeline();
                self.emit_record_done(AudioRecordingResult::Ok(self.file()));
                Continue(false)
            }
            gst::MessageView::Error(error) => {
                log::error!(
                    "Error from record bus: {:?} (debug {:?})",
                    error.error(),
                    error
                );
                self.dispose_pipeline();
                self.emit_record_done(AudioRecordingResult::Err(error.error().to_string()));
                Continue(false)
            }
            gst::MessageView::StateChanged(sc) => {
                if message.src().as_ref() == Some(self.pipeline().upcast_ref::<gst::Object>()) {
                    log::info!(
                        "Pipeline state set from {:?} -> {:?}",
                        sc.old(),
                        sc.current()
                    );
                }
                Continue(true)
            }
            _ => Continue(true),
        }
    }
}
