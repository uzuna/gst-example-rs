//! MetaMuxer
//!
//! MetaDemuxerでvideo + klvに分解されたデータをvideo + metadataに復元する
use std::sync::Mutex;

use ers_meta::ExampleRsMetaParams;
use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst::traits::PadExt;
use gst_base::prelude::AggregatorExtManual;
use gst_base::prelude::AggregatorPadExtManual;
use gst_base::subclass::prelude::{AggregatorImpl, AggregatorImplExt};
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

// AggregatorPadに流れてきたどれがvideoでどれがcapsか識別するEnum
#[derive(Debug, PartialEq, Eq)]
enum CapsType {
    Video,
    Meta,
}

#[derive(Debug)]
struct Stream {
    /// Sink pad for this stream.
    sinkpad: gst_base::AggregatorPad,
    capstype: CapsType,
}

// 一度識別したら後は同じものPadが使えるので識別した情報を保持する
#[derive(Debug, Default)]
struct State {
    streams: Vec<Stream>,
}

#[derive(Default, Debug)]
pub struct MetaMux {
    state: Mutex<State>,
}

impl MetaMux {
    // 初回の識別
    fn create_streams(&self, state: &mut State) -> Result<(), gst::FlowError> {
        for pad in self
            .obj()
            .sink_pads()
            .into_iter()
            .map(|pad| pad.downcast::<gst_base::AggregatorPad>().unwrap())
        {
            let caps = match pad.current_caps() {
                Some(caps) => caps,
                None => {
                    gst::warning!(CAT, obj: pad, "Skipping pad without caps");
                    continue;
                }
            };

            let capstype = match caps.structure(0).unwrap().name() {
                "video/x-raw" => CapsType::Video,
                "meta/x-klv" => CapsType::Meta,
                _ => unreachable!(),
            };

            state.streams.push(Stream {
                sinkpad: pad,
                capstype,
            });
        }
        Ok(())
    }

    // Aggregatorにデータが揃ってSrcに送る為にバッファをマージする
    fn drain(&self, state: &mut State) -> Result<gst::Buffer, gst::FlowError> {
        let mut buffer = None;
        for stream in state.streams.iter_mut() {
            match stream.capstype {
                CapsType::Video => {
                    gst::info!(CAT, "video");
                    buffer = Some(stream.sinkpad.pop_buffer().unwrap());
                }
                CapsType::Meta => {
                    if let Some(ref mut buffer) = buffer {
                        let metabuffer = stream.sinkpad.pop_buffer().unwrap();
                        let param: ExampleRsMetaParams = {
                            let b = metabuffer.map_readable().unwrap();
                            let v = serde_klv::from_bytes::<ExampleDataset>(b.as_slice()).unwrap();
                            v.into()
                        };
                        let wb = buffer.make_mut();
                        ers_meta::ExampleRsMeta::add(wb, param);
                    }
                }
            }
        }
        Ok(buffer.unwrap())
    }
}

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
            // フレーム順序が復元されるdecode後にマージするためvideo/x-rawに制限する
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
    // VideoとMetaの2ストリームなのでAggregatorを使う
    // 特殊な順序付けや画像同士というものでもないのでGstVideoAggregatorを使わなかった
    type ParentType = gst_base::Aggregator;
}

impl AggregatorImpl for MetaMux {
    // sinkにバッファが揃ったらここが呼ばれる
    fn aggregate(&self, timeout: bool) -> Result<gst::FlowSuccess, gst::FlowError> {
        // TODO timeoutが発生した場合の処理を決める
        gst::debug!(CAT, "aggregate timeout {}", timeout);
        let buffer = {
            let mut state = self.state.lock().unwrap();

            // Create streams
            if state.streams.is_empty() {
                self.create_streams(&mut state)?;
            }

            let all_eos = state.streams.iter().all(|stream| stream.sinkpad.is_eos());
            if all_eos {
                gst::debug!(CAT, imp: self, "All streams are EOS now");
                Err(gst::FlowError::Eos)
            } else {
                self.drain(&mut state)
            }
        }?;
        // aggregatorの場合はpushではなくfinish_bufferを使う
        // このタイミングでaggregatorがstart_streamやsegmentのイベント送信などを行う
        self.obj().finish_buffer(buffer)
    }

    // もしもソース毎にpts等が異なる場合はこれで合わせこみを行う。
    // 今回の利用ケースでは同じptsなので気にしなくて良いが記録のために残しておく
    fn clip(
        &self,
        aggregator_pad: &gst_base::AggregatorPad,
        mut buffer: gst::Buffer,
    ) -> Option<gst::Buffer> {
        {
            let buffer = buffer.make_mut();
            if let Some(segment) = aggregator_pad.segment().downcast_ref::<gst::format::Time>() {
                let pts = segment.to_running_time(buffer.pts());
                buffer.set_pts(pts);
                let dts = segment.to_running_time(buffer.dts());
                buffer.set_dts(dts);
            }
        }
        self.parent_clip(aggregator_pad, buffer)
    }

    // Aggretatorの場合はSinkからcapsが来てからsrcと再ネゴシエーションする
    // これがなければximagesinkが1x1のデフォルトで再生してしまう
    fn update_src_caps(&self, caps: &gst::Caps) -> Result<gst::Caps, gst::FlowError> {
        gst::debug!(CAT, imp: self, "update_src_caps {:?}", caps);
        // TODO create streamと重複している処理があるので共通化を検討する
        for pad in self
            .obj()
            .sink_pads()
            .into_iter()
            .map(|pad| pad.downcast::<gst_base::AggregatorPad>().unwrap())
        {
            let mut video_caps = match pad.current_caps() {
                Some(caps) => caps,
                None => {
                    gst::warning!(CAT, obj: pad, "Skipping pad without caps");
                    continue;
                }
            };
            if video_caps
                .structure(0)
                .unwrap()
                .name()
                .starts_with("video/x-raw")
            {
                video_caps.merge(caps.clone());
                gst::debug!(CAT, imp: self, "best_caps {:?}", &video_caps);
                return self.parent_update_src_caps(&video_caps);
            };
        }
        self.parent_update_src_caps(caps)
    }
}
