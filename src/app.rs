use anyhow::Error;
use gst::prelude::*;

pub(crate) fn build_appsrc_text_overlay() -> Result<gst::Pipeline, Error> {
    let pipeline = gst::Pipeline::default();
    let appsrc = gst_app::AppSrc::builder()
        .caps(&gst::Caps::new_empty_simple("text/x-raw"))
        .format(gst::Format::Time)
        .build();

    let videosrc = gst::ElementFactory::make("videotestsrc").build()?;
    let textoverlay = gst::ElementFactory::make("textoverlay").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;

    // add to pipeline and link elements
    pipeline.add_many(&[&videosrc, appsrc.upcast_ref(), &textoverlay, &sink])?;
    gst::Element::link_many(&[&textoverlay, &sink])?;

    videosrc.link(&textoverlay)?;
    appsrc.link(&textoverlay)?;

    // TODO check link
    // TODO send current timestamp 30hz cycle

    Ok(pipeline)
}
