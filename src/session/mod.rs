mod content_view;
mod manager;
mod note;
mod sidebar;

use adw::subclass::prelude::*;
use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use std::path::Path;

use self::{
    content_view::ContentView,
    manager::{LocalManager, ManagerExt},
    note::{Note, NoteExt},
    sidebar::Sidebar,
};

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/session.ui")]
    pub struct Session {
        #[template_child]
        pub sidebar: TemplateChild<Sidebar>,
        #[template_child]
        pub content_view: TemplateChild<ContentView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Session {
        const NAME: &'static str = "NwtySession";
        type Type = super::Session;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();

            Sidebar::static_type();
            ContentView::static_type();
        }
    }

    impl ObjectImpl for Session {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let note_manager = LocalManager::new(Path::new("/home/dave/Notes"));
            let notes_list = note_manager.retrive_notes().unwrap();

            self.sidebar
                .set_model(Some(&gtk::SingleSelection::new(Some(&notes_list))));

            self.sidebar
                .connect_activate(clone!(@weak obj => move |sidebar, pos| {
                    let selected_note: Note = sidebar
                        .model()
                        .unwrap()
                        .item(pos)
                        .unwrap()
                        .downcast()
                        .unwrap();

                    dbg!(selected_note.title());

                    let imp = obj.private();
                    imp.content_view.set_note(Some(&selected_note));
                }));
        }
    }

    impl WidgetImpl for Session {}
    impl BinImpl for Session {}
}

glib::wrapper! {
    pub struct Session(ObjectSubclass<imp::Session>)
        @extends gtk::Widget, adw::Bin;
}

impl Session {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Session.")
    }

    fn private(&self) -> &imp::Session {
        imp::Session::from_instance(self)
    }
}
