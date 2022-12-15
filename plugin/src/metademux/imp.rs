//! MetaDemuxer
//!
//! Videoに埋め込まれたmetadataをvideo + klvの2ストリームに分割する
use std::ops::Deref;
use std::sync::Mutex;

use ers_meta::ExampleRsMeta;
use gst::prelude::{ElementClassExt, ElementExtManual, PadExtManual};
use gst::subclass::prelude::{
    ElementImpl, ElementImplExt, GstObjectImpl, ObjectImpl, ObjectImplExt, ObjectSubclass,
    ObjectSubclassExt,
};
use gst::traits::{ElementExt, PadExt};
use gst::{glib, EventView, Segment};
use gst_base::UniqueFlowCombiner;
use once_cell::sync::Lazy;

use crate::metaklv::{ExampleDataset, KLV_CAPS};

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some(CLASS_NAME),
    )
});

#[derive(Default)]
pub struct State {
    // metaストリームをsrcと同じsegmentにするため
    segment: Segment,
}

pub struct MetaDemux {
    sinkpad: gst::Pad,
    srcpad: gst::Pad,
    klvsrcpad: Mutex<Option<gst::Pad>>,
    flow_combiner: Mutex<UniqueFlowCombiner>,
    state: Mutex<State>,
}

impl MetaDemux {
    fn sink_chain(
        &self,
        pad: &gst::Pad,
        buffer: gst::Buffer,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        gst::trace!(CAT, obj: pad, "Handling buffer {:?}", buffer);
        let res = self.sink_klv(buffer);
        gst::trace!(CAT, imp: self, "after sink_chain");
        res
    }

    fn sink_event(&self, pad: &gst::Pad, event: gst::Event) -> bool {
        gst::trace!(CAT, obj: pad, "Handling event {:?}", event);
        match event.type_() {
            // Segmentはsrcと同じものにする
            gst::EventType::Segment => {
                gst::trace!(
                    CAT,
                    obj: pad,
                    "Segment rtoffset {:?}",
                    event.running_time_offset()
                );
                if let EventView::Segment(seg) = event.view() {
                    gst::trace!(CAT, obj: pad, "Segment: {:?}", seg.segment());
                    let mut state = self.state.lock().unwrap();
                    state.segment = seg.segment().clone();
                }
                gst::Pad::event_default(pad, Some(&*self.obj()), event)
            }
            // Eosは両ストリームに配信する
            gst::EventType::Eos => gst::Pad::event_default(pad, Some(&*self.obj()), event),
            _ => self.srcpad.push_event(event),
        }
    }
    fn sink_query(&self, pad: &gst::Pad, query: &mut gst::QueryRef) -> bool {
        // この辺りのログはデバッグのため
        gst::trace!(CAT, obj: pad, "Handling query {:?}", query);
        self.srcpad.peer_query(query)
    }
    fn src_event(&self, pad: &gst::Pad, event: gst::Event) -> bool {
        gst::trace!(CAT, obj: pad, "Handling event {:?}", event);
        gst::Pad::event_default(pad, Some(&*self.obj()), event)
    }
    fn src_query(&self, pad: &gst::Pad, query: &mut gst::QueryRef) -> bool {
        gst::trace!(CAT, obj: pad, "Handling query {:?}", query);
        self.sinkpad.peer_query(query)
        // gst::Pad::query_default(pad, Some(&*self.obj()), query)
    }
    fn src_event_klv(&self, pad: &gst::Pad, event: gst::Event) -> bool {
        gst::trace!(CAT, obj: pad, "Handling klv event {:?}", event);
        gst::Pad::event_default(pad, Some(&*self.obj()), event)
    }
    fn src_query_klv(&self, pad: &gst::Pad, query: &mut gst::QueryRef) -> bool {
        gst::trace!(CAT, obj: pad, "Handling klv query {:?}", query);
        gst::Pad::query_default(pad, Some(&*self.obj()), query)
    }

    // 特定のmetaを含む場合はklvpadを生成
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
        srcpad.push_event(gst::event::StreamStart::new(&full_stream_id));
        srcpad.push_event(gst::event::Caps::new(&KLV_CAPS));

        let segment = self.state.lock().unwrap().segment.clone();
        gst::debug!(CAT, imp: self, "metapad segment ({:?})", &segment);
        srcpad.push_event(gst::event::Segment::new(&segment));
        self.obj().add_pad(&srcpad).unwrap();
        self.flow_combiner.lock().unwrap().add_pad(&srcpad);

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
                    gst::trace!(
                        CAT,
                        imp: self,
                        "videopad stream_name ({:?})",
                        self.srcpad.stream_id().unwrap()
                    );

                    srcpad
                }
            };
            let records = serde_klv::to_bytes(&ExampleDataset::from(meta.deref())).unwrap();
            let mut klvbuf = gst::Buffer::with_size(records.len()).unwrap();
            {
                let mut bw = klvbuf.make_mut().map_writable().unwrap();
                bw.copy_from_slice(&records);
            }
            // metaはsrc依存なのでsrc bufferと同じptsを指定する
            {
                let bufref = klvbuf.make_mut();
                if let Some(pts) = buffer.pts() {
                    gst::trace!(CAT, imp: self, "pts {}", pts);
                    bufref.set_pts(pts);
                }
                if let Some(dts) = buffer.dts() {
                    bufref.set_dts(dts);
                }
                if let Some(dur) = buffer.duration() {
                    bufref.set_duration(dur);
                }
                bufref.set_offset(buffer.offset());
            }
            // teeと同じくalwaysなsrcからpush
            // この後ろ次第だが基本的にはqueueを繋ぐ必要がある
            gst::trace!(CAT, imp: self, "before push video {}", buffer.offset());
            let res_src = self.srcpad.push(buffer);
            gst::trace!(CAT, imp: self, "after push srcpad");
            self.flow_combiner
                .lock()
                .unwrap()
                .update_pad_flow(&self.srcpad, res_src)?;

            // 後からmetaをpush
            // encodeなどがある場合は複数回呼ばれるためこちらの後ろにもqueueがあるのが望ましい
            gst::trace!(CAT, imp: self, "before push klv");
            let res_klv = klvpad.push(klvbuf);
            gst::trace!(CAT, imp: self, "after push klv");
            self.flow_combiner
                .lock()
                .unwrap()
                .update_pad_flow(&klvpad, res_klv)
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
            // encode後はフレームの順序が変わることがあるのでエンコード前に分離したいのでx-rawに限定する
            let caps = gst::Caps::builder("video/x-raw").build();
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
            state: Mutex::new(State::default()),
        }
    }
}
