use std::sync::Mutex;

use ers_meta::ExampleRsMeta;
use gst::glib;
use gst::prelude::{ElementClassExt, ElementExtManual, PadExtManual};
use gst::subclass::prelude::{
    ElementImpl, ElementImplExt, GstObjectImpl, ObjectImpl, ObjectImplExt, ObjectSubclass,
    ObjectSubclassExt,
};
use gst::traits::{ElementExt, PadExt};
use gst::Caps;
use gst_base::UniqueFlowCombiner;
use once_cell::sync::Lazy;

use crate::metaklv::encode_klv;

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some(CLASS_NAME),
    )
});

pub static KLV_CAPS: Lazy<Caps> = Lazy::new(|| {
    gst::Caps::builder("meta/x-klv")
        .field("parsed", true)
        .build()
});

pub struct MetaDemux {
    sinkpad: gst::Pad,
    srcpad: gst::Pad,
    klvsrcpad: Mutex<Option<gst::Pad>>,
    flow_combiner: Mutex<UniqueFlowCombiner>,
}

impl MetaDemux {
    fn sink_chain(
        &self,
        pad: &gst::Pad,
        buffer: gst::Buffer,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        gst::trace!(CAT, obj: pad, "Handling buffer {:?}", buffer);
        // let res = self.srcpad.push(buffer);
        // self.flow_combiner.lock().unwrap().update_flow(res)
        self.sink_klv(buffer)
    }

    fn sink_event(&self, pad: &gst::Pad, event: gst::Event) -> bool {
        gst::trace!(CAT, obj: pad, "Handling event {:?}", event);
        // klvにEoSとか一部は流したほうが良いのかも知れないがまだよく分かっていない
        self.srcpad.push_event(event)
    }
    fn sink_query(&self, pad: &gst::Pad, query: &mut gst::QueryRef) -> bool {
        gst::trace!(CAT, obj: pad, "Handling query {:?}", query);
        self.srcpad.peer_query(query)
    }
    fn src_event(&self, pad: &gst::Pad, event: gst::Event) -> bool {
        gst::trace!(CAT, obj: pad, "Handling event {:?}", event);
        self.sinkpad.push_event(event)
    }
    fn src_query(&self, pad: &gst::Pad, query: &mut gst::QueryRef) -> bool {
        gst::trace!(CAT, obj: pad, "Handling query {:?}", query);
        self.sinkpad.peer_query(query)
    }
    fn src_event_klv(&self, pad: &gst::Pad, event: gst::Event) -> bool {
        gst::trace!(CAT, obj: pad, "Handling klv event {:?}", event);
        gst::Pad::event_default(pad, Some(&*self.obj()), event)
    }
    fn src_query_klv(&self, pad: &gst::Pad, query: &mut gst::QueryRef) -> bool {
        gst::trace!(CAT, obj: pad, "Handling klv query {:?}", query);
        gst::Pad::query_default(pad, Some(&*self.obj()), query)
    }

    fn create_pad(&self) -> gst::Pad {
        let name = "meta";
        let templ = self.obj().element_class().pad_template(name).unwrap();
        let srcpad = gst::Pad::builder_with_template(&templ, Some(name))
            .event_function(|pad, parent, event| {
                Self::catch_panic_pad_function(parent, || false, |mt| mt.src_event_klv(pad, event))
            })
            .query_function(|pad, parent, query| {
                Self::catch_panic_pad_function(parent, || false, |mt| mt.src_query_klv(pad, query))
            })
            .build();

        srcpad.set_active(true).unwrap();

        let full_stream_id = srcpad.create_stream_id(&*self.obj(), Some(name));
        gst::debug!(CAT, imp: self, "metapad stream_name ({:?})", full_stream_id);
        let start_stream = gst::event::StreamStart::builder(&full_stream_id)
            // .flags(StreamFlags::SPARSE)
            .build();
        srcpad.push_event(start_stream);
        srcpad.push_event(gst::event::Caps::new(&KLV_CAPS));

        let segment = gst::FormattedSegment::<gst::ClockTime>::default();
        srcpad.push_event(gst::event::Segment::new(&segment));
        self.flow_combiner.lock().unwrap().add_pad(&srcpad);
        self.obj().add_pad(&srcpad).unwrap();

        srcpad
    }

    fn sink_klv(&self, buffer: gst::Buffer) -> Result<gst::FlowSuccess, gst::FlowError> {
        if let Some(meta) = buffer.meta::<ExampleRsMeta>() {
            let klvpad = {
                let mut klvpad = self.klvsrcpad.lock().unwrap();
                if let Some(ref klvpad) = *klvpad {
                    klvpad.clone()
                } else {
                    let srcpad = self.create_pad();
                    *klvpad = Some(srcpad.clone());
                    gst::info!(
                        CAT,
                        imp: self,
                        "videopad stream_name ({:?})",
                        self.srcpad.stream_id().unwrap()
                    );

                    srcpad
                }
            };
            let records = encode_klv(&meta);
            let size = klv::encode_len(&records);
            let mut klvbuf = gst::Buffer::with_size(size).unwrap();
            {
                let mut bw = klvbuf.make_mut().map_writable().unwrap();
                klv::encode(&mut bw, &records).unwrap();
            }
            {
                let bufref = klvbuf.make_mut();
                // PTSを設定するとx264encodingした場合にどこかで詰まる
                // if let Some(pts) = buffer.pts() {
                //     bufref.set_pts(pts);
                // }
                // if let Some(dts) = buffer.dts() {
                //     bufref.set_dts(dts);
                // }
                // if let Some(dur) = buffer.duration() {
                //     bufref.set_duration(dur);
                // }
                bufref.set_offset(buffer.offset());
            }
            gst::info!(CAT, imp: self, "before push klv {}", buffer.offset());
            let res_klv = klvpad.push(klvbuf);
            self.flow_combiner
                .lock()
                .unwrap()
                .update_pad_flow(&klvpad, res_klv)?;
            gst::info!(CAT, imp: self, "before push video {}", buffer.offset());
            let res_src = self.srcpad.push(buffer);
            gst::info!(CAT, imp: self, "after push srcpad");
            self.flow_combiner
                .lock()
                .unwrap()
                .update_pad_flow(&self.srcpad, res_src)
        } else {
            let res = self.srcpad.push(buffer);
            self.flow_combiner
                .lock()
                .unwrap()
                .update_pad_flow(&self.srcpad, res)
        }
    }
}

impl ElementImpl for MetaDemux {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                CLASS_NAME,
                "Demuxer",
                "Metadata Demuxer",
                "FUJINAKA Fumiya <uzuna.kf@gmail.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let caps = gst::Caps::new_any();
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let meta_pad_template = {
                gst::PadTemplate::new(
                    "meta",
                    gst::PadDirection::Src,
                    gst::PadPresence::Sometimes,
                    &KLV_CAPS,
                )
                .unwrap()
            };

            vec![src_pad_template, sink_pad_template, meta_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}

impl ObjectImpl for MetaDemux {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.add_pad(&self.sinkpad).unwrap();
        obj.add_pad(&self.srcpad).unwrap();
    }
}
impl GstObjectImpl for MetaDemux {}

#[glib::object_subclass]
impl ObjectSubclass for MetaDemux {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::MetaDemux;
    type ParentType = gst::Element;

    fn with_class(klass: &Self::Class) -> Self {
        let sinkpad = {
            let templ = klass.pad_template("sink").unwrap();
            gst::Pad::builder_with_template(&templ, Some("sink"))
                .chain_function(|pad, parent, buffer| {
                    Self::catch_panic_pad_function(
                        parent,
                        || Err(gst::FlowError::Error),
                        |mt| mt.sink_chain(pad, buffer),
                    )
                })
                .event_function(|pad, parent, event| {
                    Self::catch_panic_pad_function(parent, || false, |mt| mt.sink_event(pad, event))
                })
                .query_function(|pad, parent, query| {
                    Self::catch_panic_pad_function(parent, || false, |mt| mt.sink_query(pad, query))
                })
                .build()
        };
        let srcpad = {
            let templ = klass.pad_template("src").unwrap();
            gst::Pad::builder_with_template(&templ, Some("src"))
                .event_function(|pad, parent, event| {
                    Self::catch_panic_pad_function(parent, || false, |mt| mt.src_event(pad, event))
                })
                .query_function(|pad, parent, query| {
                    Self::catch_panic_pad_function(parent, || false, |mt| mt.src_query(pad, query))
                })
                .build()
        };
        let mut flow_combiner = UniqueFlowCombiner::new();
        flow_combiner.add_pad(&srcpad);
        Self {
            sinkpad,
            srcpad,
            klvsrcpad: Mutex::new(None),
            flow_combiner: Mutex::new(flow_combiner),
        }
    }
}
