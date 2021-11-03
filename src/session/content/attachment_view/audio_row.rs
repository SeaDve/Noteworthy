use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    glib::{self, clone, subclass::Signal},
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::unsync::OnceCell;

use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

use crate::{
    core::{AudioPlayer, PlaybackState},
    model::Attachment,
    utils::{ChainExpr, PropExpr},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-audio-row.ui")]
    pub struct AudioRow {
        #[template_child]
        pub playback_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub playback_position_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub playback_position_scale: TemplateChild<gtk::Scale>,

        pub attachment: RefCell<Attachment>,
        pub is_playing: Cell<bool>,

        pub scale_handler_id: OnceCell<glib::SignalHandlerId>,
        pub audio_player: AudioPlayer,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioRow {
        const NAME: &'static str = "NwtyContentAttachmentViewAudioRow";
        type Type = super::AudioRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("audio-row.toggle-playback", None, move |obj, _, _| {
                let is_currently_playing = obj.is_playing();
                obj.emit_by_name("playback-toggled", &[&!is_currently_playing])
                    .unwrap();
                obj.set_is_playing(!is_currently_playing);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AudioRow {
        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "playback-toggled",
                    &[bool::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "attachment",
                        "attachment",
                        "The attachment represented by this row",
                        Attachment::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-playing",
                        "Is Playing",
                        "Whether the audio file is currently playing",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
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
                "is-playing" => {
                    let is_playing = value.get().unwrap();
                    obj.set_is_playing(is_playing);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "attachment" => obj.attachment().to_value(),
                "is-playing" => obj.is_playing().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_signals();
            obj.setup_expressions();
            obj.setup_timer();
        }
    }

    impl WidgetImpl for AudioRow {}
    impl BinImpl for AudioRow {}
}

glib::wrapper! {
    pub struct AudioRow(ObjectSubclass<imp::AudioRow>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl AudioRow {
    pub fn new(attachment: &Attachment) -> Self {
        glib::Object::new(&[("attachment", attachment)]).expect("Failed to create AudioRow")
    }

    pub fn set_attachment(&self, attachment: Attachment) {
        let audio_file_uri = attachment.file().uri();
        self.audio_player().set_uri(&audio_file_uri);

        let imp = imp::AudioRow::from_instance(self);
        imp.attachment.replace(attachment);
        self.notify("attachment");
    }

    pub fn attachment(&self) -> Attachment {
        let imp = imp::AudioRow::from_instance(self);
        imp.attachment.borrow().clone()
    }

    pub fn set_is_playing(&self, is_playing: bool) {
        if is_playing {
            self.audio_player().play();
        } else {
            self.audio_player().stop();
        }
    }

    pub fn is_playing(&self) -> bool {
        let imp = imp::AudioRow::from_instance(self);
        imp.is_playing.get()
    }

    pub fn audio_player(&self) -> &AudioPlayer {
        let imp = imp::AudioRow::from_instance(self);
        &imp.audio_player
    }

    fn update_playback_display(&self) {
        if !self.is_playing() {
            return;
        }

        let imp = imp::AudioRow::from_instance(self);
        let audio_player = self.audio_player();

        if let Ok(duration) = audio_player.query_duration() {
            imp.playback_position_scale.set_range(0.0, duration as f64);
        } else {
            log::warn!("Error querying duration");
        }

        if let Ok(position) = audio_player.query_position() {
            let scale_handler_id = imp.scale_handler_id.get().unwrap();
            imp.playback_position_scale.block_signal(scale_handler_id);
            imp.playback_position_scale.set_value(position as f64);
            imp.playback_position_scale.unblock_signal(scale_handler_id);

            let seconds = position % 60;
            let minutes = (position / 60) % 60;
            let formatted_time = format!("{:02}∶{:02}", minutes, seconds);
            imp.playback_position_label.set_label(&formatted_time);
        } else {
            log::warn!("Error querying position");
        }
    }

    fn clean_playback_display(&self) {
        let imp = imp::AudioRow::from_instance(self);
        imp.playback_position_label.set_label("00∶00");

        let scale_handler_id = imp.scale_handler_id.get().unwrap();
        imp.playback_position_scale.block_signal(scale_handler_id);
        imp.playback_position_scale.set_value(0.0);
        imp.playback_position_scale.unblock_signal(scale_handler_id);
    }

    fn setup_expressions(&self) {
        let imp = imp::AudioRow::from_instance(self);

        let is_playing_expression = self.property_expression("is-playing");

        is_playing_expression.bind(
            &imp.playback_position_scale.get(),
            "sensitive",
            None::<&glib::Object>,
        );

        is_playing_expression
            .closure_expression(|args| {
                let is_playing = args[1].get().unwrap();
                if is_playing {
                    "media-playback-pause-symbolic"
                } else {
                    "media-playback-start-symbolic"
                }
            })
            .bind(
                &imp.playback_button.get(),
                "icon-name",
                None::<&glib::Object>,
            );
    }

    fn setup_signals(&self) {
        let imp = imp::AudioRow::from_instance(self);
        let scale_handler_id = imp.playback_position_scale.connect_value_changed(
            clone!(@weak self as obj => move |scale| {
                let value = scale.value();
                obj.audio_player().seek(value as u64);
            }),
        );
        imp.scale_handler_id.set(scale_handler_id).unwrap();

        imp.audio_player
            .connect_state_notify(clone!(@weak self as obj => move |audio_player,_| {
                let imp = imp::AudioRow::from_instance(&obj);

                let is_stopped = matches!(audio_player.state(), PlaybackState::Stopped);
                imp.is_playing.set(!is_stopped);

                if is_stopped {
                    obj.clean_playback_display();
                }

                obj.notify("is-playing");
            }));
    }

    fn setup_timer(&self) {
        glib::timeout_add_local(
            Duration::from_millis(500),
            clone!(@weak self as obj => @default-panic, move || {
                obj.update_playback_display();
                glib::Continue(true)
            }),
        );
    }
}
