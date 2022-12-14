//! KLVからexample-metaを復元しBufferに付与するエレメント

use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "metamux";
const CLASS_NAME: &str = "MetaMux";

mod imp;

gst::glib::wrapper! {
    pub struct MetaMux(ObjectSubclass<imp::MetaMux>) @extends gst_base::Aggregator, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        MetaMux::static_type(),
    )
}
