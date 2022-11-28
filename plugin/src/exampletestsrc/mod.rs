//! GstbinでSourceを構築する

use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "exampletestsrc";
const CLASS_NAME: &str = "ExampleTestSrc";

mod imp;

gst::glib::wrapper! {
    pub struct ExampleTestSrc(ObjectSubclass<imp::ExampleTestSrc>) @extends gst::Bin, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        ExampleTestSrc::static_type(),
    )
}
