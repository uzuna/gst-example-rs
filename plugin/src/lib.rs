//! Reference from https://gitlab.freedesktop.org/gstreamer/gst-plugins-rs

use gst::glib;

mod simpletrans;

gst::plugin_define!(
    // これはマクロなので文字列ではない。ライブラリ名と同じ文字列を指定する必要がある
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
    simpletrans::register(plugin)?;
    Ok(())
}
