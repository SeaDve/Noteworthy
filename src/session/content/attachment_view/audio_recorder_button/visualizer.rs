// Based on code from GNOME Sound Recorder
// Modified to be bidirectional and use snapshots instead of cairo
// See https://gitlab.gnome.org/GNOME/gnome-sound-recorder/-/blob/master/src/waveform.js

use gtk::{gdk, glib, graphene, prelude::*, subclass::prelude::*};

use std::cell::RefCell;

use std::collections::VecDeque;

const GUTTER: i32 = 6;
const WIDTH: f32 = 2.0;
const COLOR: gdk::RGBA = gdk::RGBA {
    red: 0.1,
    green: 0.45,
    blue: 0.8,
    alpha: 1.0,
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Visualizer {
        pub peaks: RefCell<VecDeque<f64>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Visualizer {
        const NAME: &'static str = "NwtyContentAttachmentViewAudioRecorderButtonVisualizer";
        type Type = super::Visualizer;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for Visualizer {}

    impl WidgetImpl for Visualizer {
        fn snapshot(&self, obj: &Self::Type, snapshot: &gtk::Snapshot) {
            obj.on_snapshot(snapshot);
        }
    }
}

glib::wrapper! {
    pub struct Visualizer(ObjectSubclass<imp::Visualizer>)
        @extends gtk::Widget;
}

impl Visualizer {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Visualizer")
    }

    pub fn push_peak(&self, peak: f64) {
        let imp = imp::Visualizer::from_instance(self);

        let peaks_len = imp.peaks.borrow().len() as i32;

        if peaks_len > self.allocated_width() / (2 * GUTTER) {
            imp.peaks.borrow_mut().pop_front();
        }

        imp.peaks.borrow_mut().push_back(peak);

        self.queue_draw();
    }

    pub fn clear_peaks(&self) {
        let imp = imp::Visualizer::from_instance(self);
        imp.peaks.borrow_mut().clear();

        self.queue_draw();
    }

    fn peaks(&self) -> std::cell::Ref<VecDeque<f64>> {
        let imp = imp::Visualizer::from_instance(self);
        imp.peaks.borrow()
    }

    fn on_snapshot(&self, snapshot: &gtk::Snapshot) {
        let max_height = self.allocated_height() as f32;
        let v_center = max_height / 2.0;
        let h_center = self.allocated_width() as f32 / 2.0;

        let mut pointer_a = h_center;
        let mut pointer_b = h_center;

        let peaks = self.peaks();
        let peaks_len = peaks.len();

        for (index, peak) in peaks.iter().rev().map(|peak| *peak as f32).enumerate() {
            // This reates a logarithmic decrease.
            // Starts at index 2 because log0 is undefined and log1 is 0.
            // Multiply by 6.0 to compensate on log.
            let peak_max_height = max_height.log(index as f32 + 2.0) * peak * 6.0;

            let top_point = v_center + peak_max_height;
            let this_height = -2.0 * peak_max_height;

            let rect_a = graphene::Rect::new(pointer_a, top_point, WIDTH, this_height);
            let rect_b = graphene::Rect::new(pointer_b, top_point, WIDTH, this_height);

            // Add feathering on both sides
            let mut color = COLOR;
            color.alpha = 1.0 - (index as f32 / peaks_len as f32);

            snapshot.append_color(&color, &rect_a);
            snapshot.append_color(&color, &rect_b);

            pointer_a += GUTTER as f32;
            pointer_b -= GUTTER as f32;
        }
    }
}
