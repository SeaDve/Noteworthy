use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use crate::widgets::NoteRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/notes_sidebar.ui")]
    pub struct NotesSidebar {
        #[template_child]
        pub listview: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NotesSidebar {
        const NAME: &'static str = "NwtyNotesSidebar";
        type Type = super::NotesSidebar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();

            // TODO try remove this
            NoteRow::static_type();
        }
    }

    impl ObjectImpl for NotesSidebar {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for NotesSidebar {}
    impl BinImpl for NotesSidebar {}
}

glib::wrapper! {
    pub struct NotesSidebar(ObjectSubclass<imp::NotesSidebar>)
        @extends gtk::Widget, adw::Bin;
}

impl NotesSidebar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NotesSidebar.")
    }

    pub fn set_model(&self, model: Option<&impl IsA<gtk::SelectionModel>>) {
        let imp = &imp::NotesSidebar::from_instance(self);
        imp.listview.set_model(model);
    }
}
