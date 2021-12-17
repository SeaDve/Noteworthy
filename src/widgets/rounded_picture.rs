use anyhow::Context;
use gtk::{gdk, gio, glib, graphene, gsk, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/rounded-picture.ui")]
    pub struct RoundedPicture {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,

        pub max_size: Cell<u32>,
        pub border_radius: Cell<u32>,

        pub texture: RefCell<Option<gdk::Texture>>,
        pub width: Cell<i32>,
        pub height: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RoundedPicture {
        const NAME: &'static str = "NwtyRoundedPicture";
        type Type = super::RoundedPicture;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RoundedPicture {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "file",
                        "File",
                        "The file being shown by this picture",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_uint(
                        "max-size",
                        "Max Size",
                        "The maximum size of the picture",
                        0,
                        u32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_uint(
                        "border-radius",
                        "Border Radius",
                        "The border radius of the picture",
                        0,
                        u32::MAX,
                        0,
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
                "file" => {
                    let file: Option<gio::File> = value.get().unwrap();
                    obj.set_file(file.as_ref());
                }
                "max-size" => {
                    let max_size = value.get().unwrap();
                    obj.set_max_size(max_size);
                }
                "border-radius" => {
                    let border_radius = value.get().unwrap();
                    obj.set_border_radius(border_radius);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => obj.file().to_value(),
                "max-size" => obj.max_size().to_value(),
                "border-radius" => obj.border_radius().to_value(),
                _ => unimplemented!(),
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.picture.unparent();
        }
    }

    impl WidgetImpl for RoundedPicture {
        fn measure(
            &self,
            _obj: &Self::Type,
            orientation: gtk::Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let size = if let gtk::Orientation::Horizontal = orientation {
                self.width.get()
            } else {
                self.height.get()
            };

            (size, size, -1, -1)
        }

        fn snapshot(&self, obj: &Self::Type, snapshot: &gtk::Snapshot) {
            let rounded_rect = gsk::RoundedRect::from_rect(
                graphene::Rect::new(0.0, 0.0, obj.width() as f32, obj.height() as f32),
                obj.border_radius() as f32,
            );

            snapshot.push_rounded_clip(&rounded_rect);
            self.parent_snapshot(obj, snapshot);
            snapshot.pop();
        }

        fn size_allocate(&self, _obj: &Self::Type, width: i32, height: i32, baseline: i32) {
            self.picture.allocate(width, height, baseline, None);
        }
    }
}

// TODO Consider removing this Widget and replace it with other.
// IDK why adjusting GtkPicture border-radius on css does not work now, but I had made it work before
glib::wrapper! {
    pub struct RoundedPicture(ObjectSubclass<imp::RoundedPicture>)
        @extends gtk::Widget;
}

impl RoundedPicture {
    pub fn for_file(file: &gio::File) -> Self {
        glib::Object::new(&[("file", file)]).expect("Failed to create RoundedPicture")
    }

    pub fn set_file(&self, file: Option<&gio::File>) {
        if file == self.file().as_ref() {
            return;
        }

        let imp = imp::RoundedPicture::from_instance(self);

        // TODO load lazily
        // Maybe gio::File::load_bytes_async_future then load it through
        // gdk::Texture::from_bytes in gtk 4.6
        if let Some(file) = file {
            let res = gdk::Texture::from_file(file).with_context(|| {
                format!(
                    "Failed to load texture from file at `{}`",
                    file.path().unwrap().display()
                )
            });

            match res {
                Ok(texture) => {
                    imp.picture.set_paintable(Some(&texture));
                    imp.texture.replace(Some(texture));
                    self.recompute_size();

                    log::info!("Successfully loaded picture");
                }
                Err(err) => {
                    log::error!("Error while loading texture from file: {:#}", err);
                }
            }
        } else {
            imp.picture.set_file(None::<&gio::File>);
            imp.texture.replace(None);
        }

        self.notify("file");
    }

    pub fn file(&self) -> Option<gio::File> {
        let imp = imp::RoundedPicture::from_instance(self);
        imp.picture.file()
    }

    pub fn set_max_size(&self, max_size: u32) {
        let imp = imp::RoundedPicture::from_instance(self);
        imp.max_size.set(max_size);
        self.notify("max-size");

        if imp.width.get() != 0 && imp.height.get() != 0 {
            self.recompute_size();
        }

        self.queue_resize();
    }

    pub fn max_size(&self) -> u32 {
        let imp = imp::RoundedPicture::from_instance(self);
        imp.max_size.get()
    }

    pub fn set_border_radius(&self, border_radius: u32) {
        let imp = imp::RoundedPicture::from_instance(self);
        imp.border_radius.set(border_radius);
        self.notify("border-radius");

        self.queue_draw();
    }

    pub fn border_radius(&self) -> u32 {
        let imp = imp::RoundedPicture::from_instance(self);
        imp.border_radius.get()
    }

    fn recompute_size(&self) {
        let imp = imp::RoundedPicture::from_instance(self);

        if let Some(ref texture) = *imp.texture.borrow() {
            let max_size = self.max_size() as i32;
            let width = texture.width();
            let height = texture.height();

            if width > height {
                imp.width.set(max_size);
                imp.height.set(height * max_size / width);
            } else {
                imp.width.set(width * max_size / height);
                imp.height.set(max_size);
            }
        }
    }
}
