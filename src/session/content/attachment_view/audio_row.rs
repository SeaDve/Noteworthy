use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use std::{cell::RefCell, time::Duration};

use crate::{
    core::{AudioPlayer, ClockTime, PlaybackState},
    model::Attachment,
    spawn,
    widgets::TimeLabel,
};

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-audio-row.ui")]
    pub struct AudioRow {
        #[template_child]
        pub playback_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub playback_duration_label: TemplateChild<TimeLabel>,
        #[template_child]
        pub playback_position_scale: TemplateChild<gtk::Scale>,

        pub attachment: RefCell<Attachment>,

        pub scale_handler_id: OnceCell<glib::SignalHandlerId>,
        pub seek_timeout_id: RefCell<Option<glib::SourceId>>,
        pub audio_player: AudioPlayer,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioRow {
        const NAME: &'static str = "NwtyContentAttachmentViewAudioRow";
        type Type = super::AudioRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("audio-row.toggle-playback", None, move |obj, _, _| {
                let audio_player = obj.audio_player();

                if audio_player.state() == PlaybackState::Playing {
                    audio_player.set_state(PlaybackState::Paused);
                } else {
                    audio_player.set_state(PlaybackState::Playing);
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AudioRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
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

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_signals();
            obj.setup_timer();
            obj.on_audio_player_state_changed(PlaybackState::default());
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for AudioRow {}
}

glib::wrapper! {
    pub struct AudioRow(ObjectSubclass<imp::AudioRow>)
        @extends gtk::Widget;
}

impl AudioRow {
    pub fn new(attachment: &Attachment) -> Self {
        glib::Object::new(&[("attachment", attachment)]).expect("Failed to create AudioRow")
    }

    pub fn set_attachment(&self, attachment: Attachment) {
        let audio_file_uri = attachment.file().uri();
        self.audio_player().set_uri(&audio_file_uri);

        spawn!(
            glib::PRIORITY_DEFAULT_IDLE,
            clone!(@weak self as obj => async move {
                obj.update_playback_duration_label().await;
            })
        );

        self.imp().attachment.replace(attachment);
        self.notify("attachment");
    }

    pub fn attachment(&self) -> Attachment {
        self.imp().attachment.borrow().clone()
    }

    pub fn audio_player(&self) -> &AudioPlayer {
        &self.imp().audio_player
    }

    async fn update_playback_duration_label(&self) {
        match self.audio_player().duration().await {
            Ok(duration) => {
                let imp = self.imp();

                let seconds = duration.as_secs_f64();
                imp.playback_position_scale.set_range(0.0, seconds);

                imp.playback_duration_label.set_time(duration);
            }
            Err(err) => {
                log::warn!("Error getting duration: {:?}", err);
            }
        }
    }

    fn set_playback_position_scale_value_blocking(&self, value: f64) {
        let imp = self.imp();
        let scale_handler_id = imp.scale_handler_id.get().unwrap();
        imp.playback_position_scale.block_signal(scale_handler_id);
        imp.playback_position_scale.set_value(value);
        imp.playback_position_scale.unblock_signal(scale_handler_id);
    }

    fn update_playback_position_scale(&self) {
        match self.audio_player().query_position() {
            Ok(position) => {
                self.set_playback_position_scale_value_blocking(position.as_secs_f64());
            }
            Err(err) => {
                log::warn!("Error querying position: {:?}", err);
            }
        }
    }

    fn on_audio_player_state_changed(&self, state: PlaybackState) {
        let imp = self.imp();

        match state {
            PlaybackState::Stopped | PlaybackState::Loading => {
                self.set_playback_position_scale_value_blocking(0.0);
                imp.playback_position_scale.set_sensitive(false);
            }
            PlaybackState::Playing | PlaybackState::Paused => {
                imp.playback_position_scale.set_sensitive(true);
            }
        }

        match state {
            PlaybackState::Stopped | PlaybackState::Paused | PlaybackState::Loading => {
                imp.playback_button
                    .set_icon_name("media-playback-start-symbolic");
            }
            PlaybackState::Playing => {
                imp.playback_button
                    .set_icon_name("media-playback-pause-symbolic");
            }
        }
    }

    fn on_playback_position_scale_value_changed(&self, scale: &gtk::Scale) {
        let imp = self.imp();

        // Cancel the seek when the value changed again within 20ms. So, it
        // will only seek when the value is stabilized within that span.
        if let Some(source_id) = imp.seek_timeout_id.take() {
            source_id.remove();
        }

        let value = scale.value();

        imp.seek_timeout_id
            .replace(Some(glib::timeout_add_local_once(
                Duration::from_millis(20),
                clone!(@weak self as obj => move || {
                    obj.imp().seek_timeout_id.replace(None);
                    obj.audio_player().seek(ClockTime::from_secs_f64(value));
                }),
            )));
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        let scale_handler_id = imp.playback_position_scale.connect_value_changed(
            clone!(@weak self as obj => move |scale| {
                obj.on_playback_position_scale_value_changed(scale);
            }),
        );
        imp.scale_handler_id.set(scale_handler_id).unwrap();

        imp.audio_player
            .connect_state_notify(clone!(@weak self as obj => move |audio_player| {
                obj.on_audio_player_state_changed(audio_player.state());
            }));
    }

    fn setup_timer(&self) {
        glib::timeout_add_local(
            Duration::from_millis(500),
            clone!(@weak self as obj => @default-return Continue(false), move || {
                if obj.audio_player().state() == PlaybackState::Playing {
                    obj.update_playback_position_scale();
                }

                Continue(true)
            }),
        );
    }
}
