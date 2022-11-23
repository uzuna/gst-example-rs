//! Gstbinを構築するエレメント

use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "transbin";
const CLASS_NAME: &str = "TransBin";

mod imp;

gst::glib::wrapper! {
    pub struct TransBin(ObjectSubclass<imp::TransBin>) @extends gst::Pipeline, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        TransBin::static_type(),
    )
}
