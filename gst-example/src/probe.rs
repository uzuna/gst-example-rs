use gst::{
    prelude::{
        ElementExtManual, GObjectExtManualGst, GstBinExtManual, ObjectExt, PadExtManual,
    },
    traits::{ElementExt, GstObjectExt, PadExt},
    PadProbeType, Pipeline,
};

use crate::{VideoCapsOpt, VideoTestSrcOpt};

/// run gstreamer
pub(crate) fn build_pipeline(
    testsrc: &VideoTestSrcOpt,
    videocaps: &VideoCapsOpt,
) -> Result<Pipeline, anyhow::Error> {
    gst::init()?;

    // setup element
    let pipeline = gst::Pipeline::new(None);
    let videosrc = gst::ElementFactory::make("videotestsrc").build()?;
    let h264parse = gst::ElementFactory::make("h264parse").build()?;
    let x264enc = gst::ElementFactory::make("x264enc").build()?;
    let klvsrc = gst::ElementFactory::make("klvtestsrc").build()?;
    let tsmux = gst::ElementFactory::make("mpegtsmux").build()?;
    let tsdemux = gst::ElementFactory::make("tsdemux").build()?;
    let decodebin = gst::ElementFactory::make("avdec_h264").build()?;
    let vconv = gst::ElementFactory::make("videoconvert").build()?;
    let sink = gst::ElementFactory::make("ximagesink").build()?;
    testsrc.set_properties(&videosrc);

    // add to pipeline and link elements
    pipeline.add_many(&[
        &videosrc, &klvsrc, &h264parse, &x264enc, &tsmux, &tsdemux, &decodebin, &vconv, &sink,
    ])?;
    // attach caps
    videosrc.link_filtered(&x264enc, &videocaps.get_caps())?;
    klvsrc.set_property_from_str("fps", &format!("{}", videocaps.fps));
    gst::Element::link_many(&[&tsmux, &tsdemux])?;
    gst::Element::link_many(&[&h264parse, &decodebin, &vconv, &sink])?;

    let decode_pad = h264parse.static_pad("sink").unwrap();
    tsdemux.connect_pad_added(move |src, src_pad| {
        log::info!("Received new pad {} from {}", src_pad.name(), src.name());
        if src_pad.name().contains("video") {
            src_pad.link(&decode_pad).unwrap();
        }
    });

    let vtsrc_pad = x264enc.static_pad("src").unwrap();
    let klvsrc_pad = klvsrc.static_pad("src").unwrap();
    let ts_v_pad = tsmux.request_pad_simple("sink_%d").unwrap();
    let ts_k_pad = tsmux.request_pad_simple("sink_%d").unwrap();
    vtsrc_pad.link(&ts_v_pad)?;
    klvsrc_pad.link(&ts_k_pad)?;
    klvsrc_pad.add_probe(PadProbeType::BLOCK, |_, pbi| {
        log::info!("klvsrc_pad {:?}", pbi);
        gst::PadProbeReturn::Pass
    });
    ts_k_pad.add_probe(PadProbeType::BLOCK, |_, pbi| {
        log::info!("ts_k_pad {:?}", pbi);
        gst::PadProbeReturn::Pass
    });
    Ok(pipeline)
}
