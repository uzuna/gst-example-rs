//! GstbinでSourceを構築する

use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "klvtestsrc";
const CLASS_NAME: &str = "KlvTestSrc";

mod imp;

gst::glib::wrapper! {
    pub struct KlvTestSrc(ObjectSubclass<imp::KlvTestSrc>) @extends gst_base::PushSrc, gst_base::BaseSrc, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        KlvTestSrc::static_type(),
    )
}
