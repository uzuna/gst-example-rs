//! GstbinでSourceを構築する

use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "exsrcbin";
const CLASS_NAME: &str = "ExSrcBin";

mod imp;

gst::glib::wrapper! {
    pub struct ExSrcBin(ObjectSubclass<imp::ExSrcBin>) @extends gst::Bin, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        ExSrcBin::static_type(),
    )
}
