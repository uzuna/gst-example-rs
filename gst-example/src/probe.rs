use gst::{
    prelude::{ElementExtManual, GObjectExtManualGst, GstBinExtManual, PadExtManual},
    traits::{ElementExt, GstObjectExt, PadExt},
    PadProbeData, PadProbeType, Pipeline,
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

fn show_probe(name: &str, pbd: &Option<PadProbeData>) {
    match pbd {
        Some(PadProbeData::Buffer(ref x)) => {
            log::info!("{} buffer_offset: {}", name, x.offset());
        }
        Some(_) => {}
        None => todo!(),
    }
}

/// run gstreamer
pub(crate) fn build_tee_probe(
    testsrc: &VideoTestSrcOpt,
    videocaps: &VideoCapsOpt,
    usequeue: bool,
) -> Result<Pipeline, anyhow::Error> {
    gst::init()?;

    // setup element
    let pipeline = gst::Pipeline::new(None);
    let videosrc = gst::ElementFactory::make("videotestsrc").build()?;
    let tee = gst::ElementFactory::make("tee").build()?;
    let sink = gst::ElementFactory::make("autovideosink").build()?;
    let sink2 = gst::ElementFactory::make("autovideosink").build()?;
    testsrc.set_properties(&videosrc);

    // add to pipeline and link elements
    pipeline.add_many(&[&videosrc, &tee, &sink, &sink2])?;
    // attach caps
    videosrc.link_filtered(&tee, &videocaps.get_caps())?;

    let v_src = videosrc.static_pad("src").unwrap();
    let tee_sink = tee.static_pad("sink").unwrap();

    let tee_src1 = tee.request_pad_simple("src_%u").unwrap();
    let tee_src2 = tee.request_pad_simple("src_%u").unwrap();
    let sinkpad1 = sink.static_pad("sink").unwrap();
    let sinkpad2 = sink2.static_pad("sink").unwrap();

    if usequeue {
        let queue1 = gst::ElementFactory::make("queue").build()?;
        pipeline.add_many(&[&queue1])?;
        let q1_src = queue1.static_pad("src").unwrap();
        let q1_sink = queue1.static_pad("sink").unwrap();
        tee_src1.link(&q1_sink)?;
        q1_src.link(&sinkpad1)?;
        tee_src2.link(&sinkpad2)?;

        q1_src.add_probe(PadProbeType::BLOCK, |_, pbi| {
            show_probe("q1_src", &pbi.data);
            gst::PadProbeReturn::Pass
        });
        q1_sink.add_probe(PadProbeType::BLOCK, |_, pbi| {
            show_probe("q1_sink", &pbi.data);
            gst::PadProbeReturn::Pass
        });
        tee_src1.add_probe(PadProbeType::BLOCK, |_, pbi| {
            show_probe("tee_src1", &pbi.data);
            gst::PadProbeReturn::Pass
        });
        tee_src2.add_probe(PadProbeType::BLOCK, |_, pbi| {
            show_probe("tee_src2", &pbi.data);
            gst::PadProbeReturn::Pass
        });
    } else {
        tee_src1.link(&sinkpad1)?;
        tee_src2.link(&sinkpad2)?;
        tee_src1.add_probe(PadProbeType::BLOCK, |_, pbi| {
            show_probe("tee_src1", &pbi.data);
            gst::PadProbeReturn::Pass
        });
        tee_src2.add_probe(PadProbeType::BLOCK, |_, pbi| {
            show_probe("tee_src2", &pbi.data);
            gst::PadProbeReturn::Pass
        });
    }

    v_src.add_probe(PadProbeType::BLOCK, |_, pbi| {
        show_probe("v_src", &pbi.data);
        gst::PadProbeReturn::Pass
    });
    tee_sink.add_probe(PadProbeType::BLOCK, |_, pbi| {
        show_probe("tee_sink", &pbi.data);
        gst::PadProbeReturn::Pass
    });
    sinkpad1.add_probe(PadProbeType::BLOCK, |_, pbi| {
        show_probe("sinkpad1", &pbi.data);
        gst::PadProbeReturn::Pass
    });
    sinkpad2.add_probe(PadProbeType::BLOCK, |_, pbi| {
        show_probe("sinkpad2", &pbi.data);
        gst::PadProbeReturn::Pass
    });
    Ok(pipeline)
}
