// Based on code from Obfuscate
// See https://gitlab.gnome.org/World/obfuscate/-/blob/master/src/widgets/drawing_area.rs

use gtk::{
    gdk,
    glib::{self, clone},
    graphene,
    prelude::*,
    subclass::prelude::*,
};

use std::cell::{Cell, RefCell};

use crate::core::Point;

const DEFAULT_ZOOM_LEVEL: f64 = 1.0;
const MIN_ZOOM_LEVEL: f64 = 0.1;
const MAX_ZOOM_LEVEL: f64 = 8.0;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct ScrollablePicture {
        pub paintable: RefCell<Option<gdk::Paintable>>,
        pub zoom_level: Cell<f64>,
        pub hscroll_policy: Cell<Option<gtk::ScrollablePolicy>>,
        pub hadjustment: RefCell<Option<gtk::Adjustment>>,
        pub vscroll_policy: Cell<Option<gtk::ScrollablePolicy>>,
        pub vadjustment: RefCell<Option<gtk::Adjustment>>,

        pub initial_zoom_level: Cell<f64>,
        pub initial_zoom_center: Cell<Option<Point>>,
        pub pointer_position: Cell<Option<Point>>,
        pub drag_anchor: Cell<Option<Point>>,
        pub previous_scale_factor: Cell<i32>,
        pub hadjustment_signal_id: RefCell<Option<glib::SignalHandlerId>>,
        pub vadjustment_signal_id: RefCell<Option<glib::SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScrollablePicture {
        const NAME: &'static str = "NwtyScrollablePicture";
        type Type = super::ScrollablePicture;
        type ParentType = gtk::Widget;
        type Interfaces = (gtk::Scrollable,);

        fn new() -> Self {
            Self {
                zoom_level: Cell::new(DEFAULT_ZOOM_LEVEL),
                initial_zoom_level: Cell::new(DEFAULT_ZOOM_LEVEL),
                ..Default::default()
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for ScrollablePicture {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "paintable",
                        "Paintable",
                        "Paintable shown in picture",
                        gdk::Paintable::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecDouble::new(
                        "zoom-level",
                        "Zoom Level",
                        "Current zoom level",
                        MIN_ZOOM_LEVEL,
                        MAX_ZOOM_LEVEL,
                        DEFAULT_ZOOM_LEVEL,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecOverride::for_interface::<gtk::Scrollable>("hscroll-policy"),
                    glib::ParamSpecOverride::for_interface::<gtk::Scrollable>("hadjustment"),
                    glib::ParamSpecOverride::for_interface::<gtk::Scrollable>("vscroll-policy"),
                    glib::ParamSpecOverride::for_interface::<gtk::Scrollable>("vadjustment"),
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
                "paintable" => {
                    let paintable: Option<gdk::Paintable> = value.get().unwrap();
                    obj.set_paintable(paintable.as_ref());
                }
                "zoom-level" => {
                    let zoom_level = value.get().unwrap();
                    obj.set_zoom_level(zoom_level);
                }
                "hscroll-policy" => {
                    let hscroll_policy = value.get().unwrap();
                    self.hscroll_policy.set(Some(hscroll_policy));
                }
                "hadjustment" => {
                    let hadjustment = value.get().unwrap();
                    obj.set_hadjustment_inner(hadjustment);
                }
                "vscroll-policy" => {
                    let vscroll_policy = value.get().unwrap();
                    self.vscroll_policy.set(Some(vscroll_policy));
                }
                "vadjustment" => {
                    let vadjustment = value.get().unwrap();
                    obj.set_vadjustment_inner(vadjustment);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "paintable" => obj.paintable().to_value(),
                "zoom-level" => obj.zoom_level().to_value(),
                "hscroll-policy" => obj.hscroll_policy_inner().to_value(),
                "hadjustment" => self.hadjustment.borrow().to_value(),
                "vscroll-policy" => obj.vscroll_policy_inner().to_value(),
                "vadjustment" => self.vadjustment.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.previous_scale_factor.set(obj.scale_factor());

            obj.set_overflow(gtk::Overflow::Hidden);

            obj.setup_signals();
            obj.setup_gestures();
        }
    }

    impl WidgetImpl for ScrollablePicture {
        fn measure(
            &self,
            obj: &Self::Type,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            obj.on_measure(orientation, for_size)
        }

        fn snapshot(&self, obj: &Self::Type, snapshot: &gtk::Snapshot) {
            obj.on_snapshot(snapshot);
        }

        fn size_allocate(&self, obj: &Self::Type, width: i32, height: i32, baseline: i32) {
            obj.on_size_allocate(width, height, baseline);
        }
    }

    impl ScrollableImpl for ScrollablePicture {}
}

glib::wrapper! {
    pub struct ScrollablePicture(ObjectSubclass<imp::ScrollablePicture>)
        @extends gtk::Widget,
        @implements gtk::Scrollable;
}

impl ScrollablePicture {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ScrollablePicture.")
    }

    pub fn set_paintable(&self, paintable: Option<&impl IsA<gdk::Paintable>>) {
        if paintable.map(|paintable| paintable.upcast_ref()) == self.paintable().as_ref() {
            return;
        }

        self.imp()
            .paintable
            .replace(paintable.map(|paintable| paintable.clone().upcast()));
        self.notify("paintable");
        self.set_zoom_level_to_fit();
    }

    pub fn paintable(&self) -> Option<gdk::Paintable> {
        self.imp().paintable.borrow().clone()
    }

    pub fn set_zoom_level(&self, zoom_level: f64) {
        let zoom_center =
            self.imp().pointer_position.get().unwrap_or_else(|| {
                Point::new(self.width() as f64 / 2.0, self.height() as f64 / 2.0)
            });

        self.begin_zoom(zoom_center);
        self.set_zoom_at_center(zoom_level, zoom_center);
    }

    pub fn zoom_level(&self) -> f64 {
        self.imp().zoom_level.get()
    }

    pub fn connect_zoom_level_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("zoom-level"), move |obj, _| f(obj))
    }

    pub fn zoom_in(&self) {
        self.set_zoom_level(self.zoom_level() + 0.1);
    }

    pub fn zoom_out(&self) {
        self.set_zoom_level(self.zoom_level() - 0.1);
    }

    pub fn can_zoom_out(&self) -> bool {
        self.zoom_level() > MIN_ZOOM_LEVEL
    }

    pub fn can_zoom_in(&self) -> bool {
        self.zoom_level() < MAX_ZOOM_LEVEL
    }

    /// Sets zoom level so the image is in the largest possible size without hiding any of its parts
    pub fn set_zoom_level_to_fit(&self) {
        if let Some(paintable) = self.paintable() {
            let best_fit_zoom_level = best_fit_zoom_level(
                self.width(),
                self.height(),
                paintable.intrinsic_width(),
                paintable.intrinsic_height(),
            );
            self.set_zoom_level(best_fit_zoom_level);
        }
    }

    /// Is the current zoom level at the largest possible size without hiding any of its parts
    pub fn is_zoom_level_set_to_fit(&self) -> bool {
        self.paintable().map_or(false, |paintable| {
            let best_fit_zoom_level = best_fit_zoom_level(
                self.width(),
                self.height(),
                paintable.intrinsic_width(),
                paintable.intrinsic_height(),
            );
            (self.zoom_level() - best_fit_zoom_level).abs() < f64::EPSILON
        })
    }

    fn scroll_to(&self, widget_coords: Point) {
        let hadj = self.hadjustment().unwrap();
        let vadj = self.vadjustment().unwrap();

        hadj.set_value(widget_coords.x);
        vadj.set_value(widget_coords.y);
    }

    fn is_image_movable(&self) -> bool {
        let hadj = self.hadjustment().unwrap();
        let vadj = self.vadjustment().unwrap();

        let hmovable = hadj.page_size() < hadj.upper();
        let vmovable = vadj.page_size() < vadj.upper();

        hmovable || vmovable
    }

    fn effective_zoom_level(&self) -> f64 {
        self.zoom_level() / self.scale_factor() as f64
    }

    fn hscroll_policy_inner(&self) -> gtk::ScrollablePolicy {
        self.imp()
            .hscroll_policy
            .get()
            .unwrap_or(gtk::ScrollablePolicy::Minimum)
    }

    fn vscroll_policy_inner(&self) -> gtk::ScrollablePolicy {
        self.imp()
            .vscroll_policy
            .get()
            .unwrap_or(gtk::ScrollablePolicy::Minimum)
    }

    fn begin_zoom(&self, zoom_center: Point) {
        let imp = self.imp();
        imp.initial_zoom_level.set(self.zoom_level());
        imp.initial_zoom_center
            .set(Some(self.widget_coords_to_image_coords(zoom_center)));
    }

    fn set_zoom_at_center(&self, new_zoom: f64, zoom_center: Point) {
        let imp = self.imp();

        self.imp()
            .zoom_level
            .set(new_zoom.clamp(MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL));
        self.notify("zoom-level");

        let hadj = self.hadjustment().unwrap();
        let vadj = self.vadjustment().unwrap();

        let initial_zoom_center =
            self.image_coords_to_widget_coords(imp.initial_zoom_center.get().unwrap());
        let new_scroll = Point::new(
            initial_zoom_center.x + hadj.value() - zoom_center.x,
            initial_zoom_center.y + vadj.value() - zoom_center.y,
        );
        self.scroll_to(new_scroll);

        self.queue_allocate();
    }

    fn set_hadjustment_inner(&self, hadjustment: Option<gtk::Adjustment>) {
        let imp = self.imp();

        if let Some(signal_id) = imp.hadjustment_signal_id.take() {
            if let Some(old_adjustment) = imp.hadjustment.take() {
                old_adjustment.disconnect(signal_id);
            }
        }

        if let Some(ref adjustment) = hadjustment {
            let signal_id =
                adjustment.connect_value_changed(clone!(@weak self as obj => move |_| {
                    obj.queue_draw();
                }));
            imp.hadjustment_signal_id.replace(Some(signal_id));
        }

        imp.hadjustment.replace(hadjustment);
    }

    fn set_vadjustment_inner(&self, vadjustment: Option<gtk::Adjustment>) {
        let imp = self.imp();

        if let Some(signal_id) = imp.vadjustment_signal_id.take() {
            if let Some(old_adjustment) = imp.vadjustment.take() {
                old_adjustment.disconnect(signal_id);
            }
        }

        if let Some(ref adjustment) = vadjustment {
            let signal_id =
                adjustment.connect_value_changed(clone!(@weak self as obj => move |_| {
                    obj.queue_draw();
                }));
            imp.vadjustment_signal_id.replace(Some(signal_id));
        }

        imp.vadjustment.replace(vadjustment);
    }

    fn image_coords_to_widget_coords(&self, image_coords: Point) -> Point {
        let hadj = self.hadjustment().unwrap();
        let vadj = self.vadjustment().unwrap();

        let paintable = self.paintable().unwrap();
        let zoom = self.effective_zoom_level();
        let (translate_x, translate_y) = translate(
            self.width() as f32,
            self.height() as f32,
            &paintable,
            zoom as f32,
        );

        let view_x = image_coords.x * zoom - hadj.value() + translate_x as f64;
        let view_y = image_coords.y * zoom - vadj.value() + translate_y as f64;

        Point::new(view_x, view_y)
    }

    fn widget_coords_to_image_coords(&self, view_coords: Point) -> Point {
        let hadj = self.hadjustment().unwrap();
        let vadj = self.vadjustment().unwrap();

        let paintable = self.paintable().unwrap();
        let zoom = self.effective_zoom_level();
        let (translate_x, translate_y) = translate(
            self.width() as f32,
            self.height() as f32,
            &paintable,
            zoom as f32,
        );

        let image_x = (view_coords.x + hadj.value() - translate_x as f64) / zoom;
        let image_y = (view_coords.y + vadj.value() - translate_y as f64) / zoom;

        Point::new(image_x, image_y)
    }

    fn on_measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let zoom = self.effective_zoom_level();

        let (width, height) = self.paintable().map_or((300, 300), |paintable| {
            (
                (paintable.intrinsic_width() as f64 * zoom) as i32,
                (paintable.intrinsic_height() as f64 * zoom) as i32,
            )
        });

        if orientation == gtk::Orientation::Horizontal {
            (0, width, -1, -1)
        } else {
            (0, height, -1, -1)
        }
    }

    fn on_snapshot(&self, snapshot: &gtk::Snapshot) {
        if let Some(paintable) = self.paintable() {
            let hadj = self.hadjustment().unwrap();
            let vadj = self.vadjustment().unwrap();
            let zoom = self.effective_zoom_level() as f32;
            let (translate_x, translate_y) =
                translate(self.width() as f32, self.height() as f32, &paintable, zoom);

            snapshot.save();
            snapshot.translate(&graphene::Point::new(
                (-hadj.value() as f32 + translate_x).round(),
                (-vadj.value() as f32 + translate_y).round(),
            ));
            snapshot.scale(zoom, zoom);

            paintable.snapshot(
                snapshot.upcast_ref::<gdk::Snapshot>(),
                paintable.intrinsic_width() as f64,
                paintable.intrinsic_height() as f64,
            );
            snapshot.restore();
        }
    }

    fn on_size_allocate(&self, width: i32, height: i32, _baseline: i32) {
        if let Some(paintable) = self.paintable() {
            let hadj = self.hadjustment().unwrap();
            let vadj = self.vadjustment().unwrap();

            let zoom = self.effective_zoom_level();

            hadj.configure(
                hadj.value(),
                0.0,
                (width as f64).max(paintable.intrinsic_width() as f64 * zoom),
                0.1 * width as f64,
                0.9 * width as f64,
                width as f64,
            );
            vadj.configure(
                vadj.value(),
                0.0,
                (height as f64).max(paintable.intrinsic_height() as f64 * zoom),
                0.1 * height as f64,
                0.9 * height as f64,
                height as f64,
            );
        }
    }

    fn setup_signals(&self) {
        self.connect_scale_factor_notify(|obj| {
            let imp = obj.imp();

            let change = obj.scale_factor() as f64 / imp.previous_scale_factor.get() as f64;
            imp.zoom_level.set(obj.zoom_level() * change);
            obj.notify("zoom-level");

            imp.previous_scale_factor.set(obj.scale_factor());

            obj.queue_allocate();
        });
    }

    fn setup_gestures(&self) {
        let gesture_zoom = gtk::GestureZoom::new();
        gesture_zoom.connect_begin(clone!(@weak self as obj => move |gesture, _| {
            let view_center = Point::from_tuple(gesture.bounding_box_center().unwrap());
            obj.begin_zoom(view_center);

            gesture.set_state(gtk::EventSequenceState::Claimed);
        }));
        gesture_zoom.connect_scale_changed(clone!(@weak self as obj => move |gesture, scale| {
            let view_center = Point::from_tuple(gesture.bounding_box_center().unwrap());
            obj.set_zoom_at_center(obj.imp().initial_zoom_level.get() * scale, view_center);
        }));
        self.add_controller(&gesture_zoom);

        let gesture_drag = gtk::GestureDrag::new();
        gesture_drag.connect_drag_begin(clone!(@weak self as obj => move |_, _, _| {
            let hadj = obj.hadjustment().unwrap();
            let vadj = obj.vadjustment().unwrap();

            obj.imp().drag_anchor.set(Some(Point::new(hadj.value() as f64, vadj.value() as f64)));

            if obj.is_image_movable() {
                if let Some(cursor) = gdk::Cursor::from_name("move", None) {
                    obj.set_cursor(Some(&cursor));
                }
            }
        }));
        gesture_drag.connect_drag_update(
            clone!(@weak self as obj => move |_, offset_x, offset_y| {
                let drag_anchor = obj.imp().drag_anchor.get().unwrap();
                obj.scroll_to(Point::new(drag_anchor.x - offset_x, drag_anchor.y - offset_y));
            }),
        );
        gesture_drag.connect_drag_end(clone!(@weak self as obj => move |_, _, _| {
            obj.imp().drag_anchor.set(None);
            obj.set_cursor(None);
        }));
        self.add_controller(&gesture_drag);

        let motion_controller = gtk::EventControllerMotion::new();
        motion_controller.connect_enter(clone!(@weak self as obj => move |_, x, y| {
            obj.imp().pointer_position.set(Some(Point::new(x, y)));
        }));
        motion_controller.connect_motion(clone!(@weak self as obj => move |_, x, y| {
            obj.imp().pointer_position.set(Some(Point::new(x, y)));
        }));
        motion_controller.connect_leave(clone!(@weak self as obj => move |_| {
            obj.imp().pointer_position.set(None);
        }));
        self.add_controller(&motion_controller);

        let scroll_controller =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll_controller.connect_scroll(
            clone!(@weak self as obj => @default-panic, move |event, _delta_x, delta_y| {
                if event.current_event_state().contains(gdk::ModifierType::CONTROL_MASK) {
                    if delta_y as i32 > 0 {
                        obj.zoom_out();
                    } else {
                        obj.zoom_in();
                    }
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            }),
        );
        self.add_controller(&scroll_controller);
    }
}

impl Default for ScrollablePicture {
    fn default() -> Self {
        Self::new()
    }
}

// https://gitlab.gnome.org/GNOME/eog/-/blob/master/src/zoom.c
fn zoom_fit_size(dest_width: i32, dest_height: i32, src_width: i32, src_height: i32) -> (i32, i32) {
    if src_width == 0 || src_height == 0 {
        return (0, 0);
    }

    let height = (src_height * dest_width) as f64 / src_width as f64 + 0.5;

    if height > dest_height as f64 {
        let width = (src_width * dest_height) as f64 / src_height as f64 + 0.5;
        return (width.floor() as i32, dest_height);
    }

    (dest_width, height.floor() as i32)
}

// https://gitlab.gnome.org/GNOME/eog/-/blob/master/src/zoom.c
fn best_fit_zoom_level(dest_width: i32, dest_height: i32, src_width: i32, src_height: i32) -> f64 {
    if src_width == 0 || src_height == 0 {
        return 1.0;
    }

    if dest_width == 0 || dest_height == 0 {
        return 0.0;
    }

    let (width, height) = zoom_fit_size(dest_width, dest_height, src_width, src_height);

    let w_factor = width as f64 / src_width as f64;
    let h_factor = height as f64 / src_height as f64;

    w_factor.min(h_factor)
}

fn translate(width: f32, height: f32, paintable: &gdk::Paintable, zoom: f32) -> (f32, f32) {
    let (mut translate_x, mut translate_y) = (0.0, 0.0);
    let paintable_width = paintable.intrinsic_width() as f32 * zoom;
    let paintable_height = paintable.intrinsic_height() as f32 * zoom;

    if width > paintable_width {
        translate_x = (width - paintable_width) / 2.0;
    }
    if height > paintable_height {
        translate_y = (height - paintable_height) / 2.0;
    }
    (translate_x, translate_y)
}
