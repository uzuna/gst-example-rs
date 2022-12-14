use ers_meta::ExampleRsMetaParams;
use gst::glib;
use gst::prelude::Cast;
use gst::prelude::ElementExtManual;
use gst::prelude::PadExtManual;
use gst::prelude::StaticType;
use gst::subclass::prelude::*;
use gst::traits::PadExt;
use gst_base::subclass::prelude::AggregatorImpl;
use gst_base::subclass::prelude::AggregatorImplExt;
use gst_base::traits::AggregatorPadExt;
use once_cell::sync::Lazy;

use crate::metaklv::ExampleDataset;

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some(CLASS_NAME),
    )
});

#[derive(Default, Debug)]
pub struct MetaMux {}

impl ElementImpl for MetaMux {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                CLASS_NAME,
                "Muxer",
                "Metadata Muxer",
                "FUJINAKA Fumiya <uzuna.kf@gmail.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &[gst::Structure::builder("video/x-raw").build()]
                    .into_iter()
                    .collect::<gst::Caps>(),
            )
            .unwrap();

            let sink_pad_template = gst::PadTemplate::with_gtype(
                "sink_%u",
                gst::PadDirection::Sink,
                gst::PadPresence::Request,
                &[
                    gst::Structure::builder("video/x-raw").build(),
                    gst::Structure::builder("meta/x-klv")
                        .field("parsed", true)
                        .build(),
                ]
                .into_iter()
                .collect::<gst::Caps>(),
                gst_base::AggregatorPad::static_type(),
            )
            .unwrap();

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}

impl ObjectImpl for MetaMux {}
impl GstObjectImpl for MetaMux {}

#[glib::object_subclass]
impl ObjectSubclass for MetaMux {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::MetaMux;
    type ParentType = gst_base::Aggregator;
}

impl AggregatorImpl for MetaMux {
    fn aggregate(&self, timeout: bool) -> Result<gst::FlowSuccess, gst::FlowError> {
        gst::debug!(CAT, "aggregate timeout {}", timeout);
        let mut video_buf = None;
        // TODO impl aggregate
        for pad in self
            .obj()
            .sink_pads()
            .into_iter()
            .map(|pad| pad.downcast::<gst_base::AggregatorPad>().unwrap())
        {
            if let Some(buf) = pad.pop_buffer() {
                gst::debug!(
                    CAT,
                    "aggregate timeout has buf {:?} {:?} {:?} {:?}",
                    pad.caps(),
                    buf.pts(),
                    buf.dts(),
                    buf.offset()
                );
                match pad.caps().unwrap().structure(0).unwrap().name() {
                    "video/x-raw" => video_buf = Some(buf),
                    "meta/x-klv" => {
                        if let Some(ref mut video_buf) = video_buf {
                            let param: ExampleRsMetaParams = {
                                let b = buf.map_readable().unwrap();
                                let v = serde_klv::from_bytes::<ExampleDataset>(&b).unwrap();
                                v.into()
                            };
                            let wb = video_buf.make_mut();
                            ers_meta::ExampleRsMeta::add(wb, param);
                        }
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            }
        }
        if let Some(buffer) = video_buf {
            self.obj().src_pads()[0].push(buffer)
        } else {
            Err(gst::FlowError::NotSupported)
        }
    }

    fn create_new_pad(
        &self,
        templ: &gst::PadTemplate,
        req_name: Option<&str>,
        caps: Option<&gst::Caps>,
    ) -> Option<gst_base::AggregatorPad> {
        self.parent_create_new_pad(templ, req_name, caps)
    }
}
