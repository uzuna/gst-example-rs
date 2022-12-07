use gst::glib;
use gst::subclass::prelude::{ElementImpl, GstObjectImpl, ObjectImpl, ObjectSubclass};
use gst::Caps;
use once_cell::sync::Lazy;

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some(CLASS_NAME),
    )
});

pub static KLV_CAPS: Lazy<Caps> = Lazy::new(|| {
    gst::Caps::builder("meta/x-klv")
        .field("parsed", true)
        .build()
});

#[derive(Default)]
pub struct MetaDemux {}

impl ElementImpl for MetaDemux {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                CLASS_NAME,
                "Demuxer",
                "Metadata Demuxer",
                "FUJINAKA Fumiya <uzuna.kf@gmail.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let caps = gst::Caps::new_any();
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let meta_pad_template = {
                gst::PadTemplate::new(
                    "meta",
                    gst::PadDirection::Src,
                    gst::PadPresence::Sometimes,
                    &KLV_CAPS,
                )
                .unwrap()
            };

            vec![src_pad_template, sink_pad_template, meta_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}
impl ObjectImpl for MetaDemux {}
impl GstObjectImpl for MetaDemux {}

#[glib::object_subclass]
impl ObjectSubclass for MetaDemux {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::MetaDemux;
    type ParentType = gst::Element;
}
