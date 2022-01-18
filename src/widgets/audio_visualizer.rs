// Based on code from GNOME Sound Recorder GPLv3
// Modified to be bidirectional and use snapshots instead of cairo
// See https://gitlab.gnome.org/GNOME/gnome-sound-recorder/-/blob/master/src/waveform.js

use gtk::{gdk, glib, graphene, gsk, prelude::*, subclass::prelude::*};

use std::{cell::RefCell, collections::VecDeque};

const GUTTER: f32 = 6.0;
const LINE_WIDTH: f32 = 3.0;
const LINE_RADIUS: f32 = 8.0;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AudioVisualizer {
        pub peaks: RefCell<VecDeque<f32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioVisualizer {
        const NAME: &'static str = "NwtyAudioVisualizer";
        type Type = super::AudioVisualizer;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("audiovisualizer");
        }
    }

    impl ObjectImpl for AudioVisualizer {}

    impl WidgetImpl for AudioVisualizer {
        fn snapshot(&self, obj: &Self::Type, snapshot: &gtk::Snapshot) {
            obj.on_snapshot(snapshot);
        }
    }
}

glib::wrapper! {
    pub struct AudioVisualizer(ObjectSubclass<imp::AudioVisualizer>)
        @extends gtk::Widget;
}

impl AudioVisualizer {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AudioVisualizer")
    }

    pub fn push_peak(&self, peak: f32) {
        let mut peaks = self.peaks_mut();

        if peaks.len() as i32 > self.allocated_width() / (2 * GUTTER as i32) {
            peaks.pop_front();
        }

        peaks.push_back(peak);

        self.queue_draw();
    }

    pub fn clear_peaks(&self) {
        self.peaks_mut().clear();

        self.queue_draw();
    }

    fn peaks(&self) -> std::cell::Ref<VecDeque<f32>> {
        self.imp().peaks.borrow()
    }

    fn peaks_mut(&self) -> std::cell::RefMut<VecDeque<f32>> {
        self.imp().peaks.borrow_mut()
    }

    fn on_snapshot(&self, snapshot: &gtk::Snapshot) {
        let width = self.width() as f32;
        let height = self.height() as f32;

        let h_center = width as f32 / 2.0;
        let v_center = height / 2.0;

        let mut pointer_a = h_center;
        let mut pointer_b = h_center;

        let clear = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
        let color = self.style_context().color();

        for (index, peak) in self.peaks().iter().rev().enumerate() {
            // This makes both sides decrease logarithmically.
            // Starts at index 2 because log0 is undefined and log1 is 0.
            // Multiply by 2.5 to compensate on log.
            let peak_max_height = height.log(index as f32 + 2.0) * peak * 2.8;

            let top_point = v_center + peak_max_height;
            let this_height = -2.0 * peak_max_height;

            let rect_a = graphene::Rect::new(pointer_a, top_point, LINE_WIDTH, this_height);
            let rect_b = graphene::Rect::new(pointer_b, top_point, LINE_WIDTH, this_height);

            pointer_a -= GUTTER;
            pointer_b += GUTTER;

            snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect_a, LINE_RADIUS));
            snapshot.append_linear_gradient(
                &graphene::Rect::new(0.0, 0.0, h_center, height),
                &graphene::Point::new(0.0, v_center),
                &graphene::Point::new(h_center, v_center),
                &[
                    gsk::ColorStop::new(0.0, clear),
                    gsk::ColorStop::new(1.0, color),
                ],
            );
            snapshot.pop();

            snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect_b, LINE_RADIUS));
            snapshot.append_linear_gradient(
                &graphene::Rect::new(h_center, 0.0, h_center, height),
                &graphene::Point::new(width, v_center),
                &graphene::Point::new(h_center, v_center),
                &[
                    gsk::ColorStop::new(0.0, clear),
                    gsk::ColorStop::new(1.0, color),
                ],
            );
            snapshot.pop();
        }
    }
}
