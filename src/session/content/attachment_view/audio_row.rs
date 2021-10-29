use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    glib::{self, subclass::Signal},
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::{Cell, RefCell};

use crate::{
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

        pub attachment: RefCell<Attachment>,
        pub is_playing: Cell<bool>,
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
                        glib::ParamFlags::READWRITE,
                    ),
                ]
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
                "is-playing" => {
                    let is_playing = value.get().unwrap();
                    self.is_playing.set(is_playing);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "attachment" => self.attachment.borrow().to_value(),
                "is-playing" => self.is_playing.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_expressions();
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

    pub fn connect_playback_toggled<F: Fn(&Self, bool) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("playback-toggled", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let is_active = values[1].get::<bool>().unwrap();
            f(&obj, is_active);
            None
        })
        .unwrap()
    }

    pub fn uri(&self) -> String {
        let attachment: Attachment = self.property("attachment").unwrap().get().unwrap();
        attachment.file().uri().into()
    }

    pub fn set_is_playing(&self, is_playing: bool) {
        self.set_property("is-playing", is_playing).unwrap();
    }

    fn is_playing(&self) -> bool {
        self.property("is-playing").unwrap().get().unwrap()
    }

    fn setup_expressions(&self) {
        let imp = imp::AudioRow::from_instance(self);

        self.property_expression("is-playing")
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
}
