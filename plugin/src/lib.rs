//! Reference from https://gitlab.freedesktop.org/gstreamer/gst-plugins-rs

use gst::glib;

mod exampletestsrc;
mod klvtestsrc;
mod metademux;
mod metaklv;
mod metatrans;
mod testtrans;

gst::plugin_define!(
    // use the same name as [lib.name]
    rsexample,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    // The licence parameter must be one of: LGPL, GPL, QPL, GPL/QPL, MPL, BSD, MIT/X11, Proprietary, unknown.
    // refer: https://api.gtkd.org/gstreamer.c.types.GstPluginDesc.html
    "MIT/X11",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    testtrans::register(plugin)?;
    metatrans::register(plugin)?;
    exampletestsrc::register(plugin)?;
    klvtestsrc::register(plugin)?;
    metademux::register(plugin)?;
    Ok(())
}
