//! example-metaを制御するためのエレメント

use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "metademux";
const CLASS_NAME: &str = "MetaDemux";

mod imp;

gst::glib::wrapper! {
    pub struct MetaDemux(ObjectSubclass<imp::MetaDemux>) @extends gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        MetaDemux::static_type(),
    )
}
