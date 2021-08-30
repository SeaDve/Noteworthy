mod note_row;

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use note_row::NoteRow;

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

    fn private(&self) -> &imp::NotesSidebar {
        imp::NotesSidebar::from_instance(self)
    }
}
