use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "testtrans";
const CLASS_NAME: &str = "TestTrans";

mod imp;

gst::glib::wrapper! {
    pub struct TestTrans(ObjectSubclass<imp::TestTrans>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        TestTrans::static_type(),
    )
}
