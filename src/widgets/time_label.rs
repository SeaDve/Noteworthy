use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::Cell;

use crate::core::ClockTime;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/time-label.ui")]
    pub struct TimeLabel {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        pub time: Cell<ClockTime>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TimeLabel {
        const NAME: &'static str = "NwtyTimeLabel";
        type Type = super::TimeLabel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TimeLabel {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_boxed(
                    "time",
                    "Time",
                    "Time being shown by label",
                    ClockTime::static_type(),
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
                "time" => {
                    let time = value.get().unwrap();
                    obj.set_time(time);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "time" => obj.time().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            obj.reset();
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.label.unparent();
        }
    }

    impl WidgetImpl for TimeLabel {}
}

glib::wrapper! {
    pub struct TimeLabel(ObjectSubclass<imp::TimeLabel>)
        @extends gtk::Widget;
}

impl TimeLabel {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create TimeLabel")
    }

    pub fn set_time(&self, time: ClockTime) {
        let imp = imp::TimeLabel::from_instance(self);

        let seconds = time.as_secs();
        let seconds_display = seconds % 60;
        let minutes_display = seconds / 60;
        let formatted_time = format!("{:02}∶{:02}", minutes_display, seconds_display);
        imp.label.set_label(&formatted_time);

        imp.time.set(time);
        self.notify("time");
    }

    pub fn time(&self) -> ClockTime {
        let imp = imp::TimeLabel::from_instance(self);
        imp.time.get()
    }

    pub fn reset(&self) {
        self.set_time(ClockTime::ZERO);
    }
}
