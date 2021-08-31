use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/note_row.ui")]
    pub struct NoteRow {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteRow {
        const NAME: &'static str = "NwtyNoteRow";
        type Type = super::NoteRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NoteRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "title",
                        "Title",
                        "Title of this row",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_string(
                        "subtitle",
                        "Subitle",
                        "Subtitle of this row",
                        None,
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
                "title" => {
                    let title = value.get().unwrap();
                    self.title.set_label(title);
                }
                "subtitle" => {
                    let subtitle: &str = value.get().unwrap();
                    self.subtitle.set_label(subtitle.lines().next().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.title.label().to_value(),
                "subtitle" => self.subtitle.label().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for NoteRow {}
    impl BinImpl for NoteRow {}
}

glib::wrapper! {
    pub struct NoteRow(ObjectSubclass<imp::NoteRow>)
        @extends gtk::Widget, adw::Bin;
}

impl NoteRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NoteRow.")
    }
}
