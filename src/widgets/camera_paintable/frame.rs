use gtk::{gdk, glib, prelude::*};

#[derive(Debug)]
pub struct Frame(pub gst_video::VideoFrame<gst_video::video_frame::Readable>);

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        self.0.plane_data(0).unwrap()
    }
}

impl From<Frame> for gdk::Paintable {
    fn from(frame: Frame) -> gdk::Paintable {
        let format = match frame.0.format() {
            gst_video::VideoFormat::Bgra => gdk::MemoryFormat::B8g8r8a8,
            gst_video::VideoFormat::Argb => gdk::MemoryFormat::A8r8g8b8,
            gst_video::VideoFormat::Rgba => gdk::MemoryFormat::R8g8b8a8,
            gst_video::VideoFormat::Abgr => gdk::MemoryFormat::A8b8g8r8,
            gst_video::VideoFormat::Rgb => gdk::MemoryFormat::R8g8b8,
            gst_video::VideoFormat::Bgr => gdk::MemoryFormat::B8g8r8,
            other => unreachable!("Invalid, video_format: {}", other),
        };

        let width = frame.width() as i32;
        let height = frame.height() as i32;
        let rowstride = frame.0.plane_stride()[0] as usize;

        gdk::MemoryTexture::new(
            width,
            height,
            format,
            &glib::Bytes::from_owned(frame),
            rowstride,
        )
        .upcast()
    }
}

impl Frame {
    pub fn new(buffer: &gst::Buffer, info: &gst_video::VideoInfo) -> Self {
        Self(gst_video::VideoFrame::from_buffer_readable(buffer.clone(), info).unwrap())
    }

    pub fn width(&self) -> u32 {
        self.0.width()
    }

    pub fn height(&self) -> u32 {
        self.0.height()
    }
}
