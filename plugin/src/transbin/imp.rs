

use gst::glib;
use gst::subclass::prelude::*;
use once_cell::sync::Lazy;

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some("TransBin"),
    )
});
#[derive(Default)]
pub struct TransBin {}

impl TransBin {}

impl ElementImpl for TransBin {}
impl ObjectImpl for TransBin {}
impl GstObjectImpl for TransBin {}

#[glib::object_subclass]
impl ObjectSubclass for TransBin {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::TransBin;
    type ParentType = gst::Pipeline;
}

impl BinImpl for TransBin {}
impl PipelineImpl for TransBin {}