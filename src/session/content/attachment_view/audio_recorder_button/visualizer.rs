// Based on code from GNOME Sound Recorder
// Modified to be bidirectional
// See https://gitlab.gnome.org/GNOME/gnome-sound-recorder/-/blob/master/src/waveform.js

use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    cairo,
    glib::{self, clone},
    subclass::prelude::*,
};

use std::cell::RefCell;

use std::collections::VecDeque;

const GUTTER: i32 = 6;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Visualizer {
        pub peaks: RefCell<VecDeque<f64>>,

        pub drawing_area: gtk::DrawingArea,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Visualizer {
        const NAME: &'static str = "NwtyContentAttachmentViewAudioRecorderButtonVisualizer";
        type Type = super::Visualizer;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for Visualizer {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_drawing_area();
        }
    }

    impl WidgetImpl for Visualizer {}
    impl BinImpl for Visualizer {}
}

glib::wrapper! {
    pub struct Visualizer(ObjectSubclass<imp::Visualizer>)
        @extends gtk::Widget, adw::Bin;
}

impl Visualizer {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Visualizer")
    }

    pub fn push_peak(&self, peak: f64) {
        let imp = imp::Visualizer::from_instance(self);

        let peaks_len = imp.peaks.borrow().len() as i32;

        if peaks_len > imp.drawing_area.allocated_width() / (2 * GUTTER) {
            imp.peaks.borrow_mut().pop_front();
        }

        imp.peaks.borrow_mut().push_back(peak);

        imp.drawing_area.queue_draw();
    }

    pub fn clear_peaks(&self) {
        let imp = imp::Visualizer::from_instance(self);
        imp.peaks.borrow_mut().clear();
    }

    fn peaks(&self) -> std::cell::Ref<VecDeque<f64>> {
        let imp = imp::Visualizer::from_instance(self);
        imp.peaks.borrow()
    }

    fn setup_drawing_area(&self) {
        let imp = imp::Visualizer::from_instance(self);

        imp.drawing_area
            .set_draw_func(clone!(@weak self as obj => move |da,ctx,_,_| {
                obj.drawing_area_draw(da, ctx);
            }));

        self.set_child(Some(&imp.drawing_area));
    }

    fn drawing_area_draw(&self, da: &gtk::DrawingArea, ctx: &cairo::Context) {
        let max_height = da.allocated_height() as f64;
        let v_center = max_height / 2.0;
        let h_center = da.allocated_width() as f64 / 2.0;

        // 1.5 is to avoid overlapping lines at the middle
        let mut pointer_a = h_center + 2.5;
        let mut pointer_b = h_center - 2.5;

        let peaks = self.peaks();
        let peaks_len = peaks.len();

        ctx.set_line_cap(cairo::LineCap::Round);
        ctx.set_line_width(2.0);

        for (index, peak) in peaks.iter().rev().enumerate() {
            // Add feathering on both sides
            let alpha = 1.0 - (index as f64 / peaks_len as f64);
            ctx.set_source_rgba(0.1, 0.45, 0.8, alpha);

            // Creates a logarithmic decrease
            // Starts at index 2 because log0 is undefined and log1 is 0
            let this_max_height = max_height.log(index as f64 + 2.0) * 10.0;

            ctx.move_to(pointer_a, v_center + peak * this_max_height);
            ctx.line_to(pointer_a, v_center - peak * this_max_height);
            ctx.stroke().unwrap();

            ctx.move_to(pointer_b, v_center + peak * this_max_height);
            ctx.line_to(pointer_b, v_center - peak * this_max_height);
            ctx.stroke().unwrap();

            pointer_a += GUTTER as f64;
            pointer_b -= GUTTER as f64;
        }
    }
}
