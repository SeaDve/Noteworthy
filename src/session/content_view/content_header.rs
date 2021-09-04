use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content_header.ui")]
    pub struct ContentHeader {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContentHeader {
        const NAME: &'static str = "NwtyContentHeader";
        type Type = super::ContentHeader;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContentHeader {
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
                    let subtitle = value.get().unwrap();
                    self.subtitle.set_label(subtitle);
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

    impl WidgetImpl for ContentHeader {}
    impl BoxImpl for ContentHeader {}
}

glib::wrapper! {
    pub struct ContentHeader(ObjectSubclass<imp::ContentHeader>)
        @extends gtk::Widget, gtk::Box;
}

impl ContentHeader {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ContentHeader.")
    }

    pub fn set_title(&self, title: &str) {
        self.set_property("title", title).unwrap();
    }

    pub fn title(&self) -> String {
        self.property("title").unwrap().get().unwrap()
    }
}
