mod note_row;

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use note_row::NoteRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/notes_sidebar.ui")]
    pub struct Sidebar {
        #[template_child]
        pub listview: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "NwtySidebar";
        type Type = super::Sidebar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();

            NoteRow::static_type();
        }
    }

    impl ObjectImpl for Sidebar {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for Sidebar {}
    impl BinImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget, adw::Bin;
}

impl Sidebar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Sidebar.")
    }

    pub fn set_model(&self, model: Option<&impl IsA<gtk::SelectionModel>>) {
        let imp = self.private();
        imp.listview.set_model(model);
    }

    pub fn model(&self) -> Option<impl IsA<gtk::SelectionModel>> {
        let imp = self.private();
        imp.listview.model()
    }

    pub fn connect_activate(
        &self,
        f: impl Fn(&gtk::ListView, u32) + 'static,
    ) -> glib::SignalHandlerId {
        let imp = self.private();
        imp.listview.connect_activate(f)
    }

    fn private(&self) -> &imp::Sidebar {
        imp::Sidebar::from_instance(self)
    }
}
