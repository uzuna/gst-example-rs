//! example-metaを制御するためのエレメント

use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "metatrans";
const CLASS_NAME: &str = "MetaTrans";

mod imp;

gst::glib::wrapper! {
    pub struct MetaTrans(ObjectSubclass<imp::MetaTrans>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        MetaTrans::static_type(),
    )
}
