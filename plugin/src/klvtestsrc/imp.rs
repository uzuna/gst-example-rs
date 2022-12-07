use std::sync::atomic::AtomicU64;
use std::sync::{Mutex, RwLock};

use ers_meta::ExampleRsMetaParams;
use gst::glib::ParamFlags;
use gst::prelude::{
    ClockExtManual, Displayable, GstParamSpecBuilderExt, OptionAdd, ParamSpecBuilderExt, ToValue,
};
use gst::subclass::prelude::*;
use gst::traits::{ClockExt, ElementExt};
use gst::{glib, Caps, Fraction};

use gst_base::prelude::BaseSrcExtManual;
use gst_base::subclass::base_src::CreateSuccess;
use gst_base::subclass::prelude::{BaseSrcImpl, PushSrcImpl};
use gst_base::traits::BaseSrcExt;

use once_cell::sync::Lazy;

use crate::metaklv::encode_klv_params;

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

// properties()でクラス定数を参照できないので外部に定義
const DEFAULT_IS_LIVE: bool = false;

struct Settings {
    fps: Fraction,
    is_live: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: Fraction::new(30, 1),
            is_live: DEFAULT_IS_LIVE,
        }
    }
}

struct ClockWait {
    clock_id: Option<gst::SingleShotClockId>,
    // flushing: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for ClockWait {
    fn default() -> ClockWait {
        ClockWait {
            clock_id: None,
            // flushing: true,
        }
    }
}

#[derive(Default)]
pub struct KlvTestSrc {
    count: AtomicU64,
    clock_wait: Mutex<ClockWait>,
    settings: RwLock<Settings>,
}

impl KlvTestSrc {}

impl ElementImpl for KlvTestSrc {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                CLASS_NAME,
                "Generic",
                "Generate Test Metadata",
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
                &KLV_CAPS,
            )
            .unwrap();

            vec![src_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
    fn change_state(
        &self,
        transition: gst::StateChange,
    ) -> Result<gst::StateChangeSuccess, gst::StateChangeError> {
        gst::debug!(CAT, imp: self, "change_state {}", &transition);

        if let gst::StateChange::ReadyToPaused = transition {
            self.obj().set_live(self.settings.read().unwrap().is_live);
        }

        self.parent_change_state(transition)
    }
}

impl ObjectImpl for KlvTestSrc {
    // Called right after construction of a new instance
    fn constructed(&self) {
        // Call the parent class' ::constructed() implementation first
        self.parent_constructed();

        let obj = self.obj();
        // Initialize live-ness and notify the base class that
        // we'd like to operate in Time format
        obj.set_live(DEFAULT_IS_LIVE);
        obj.set_format(gst::Format::Time);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                gst::param_spec::ParamSpecFraction::builder("fps")
                    .nick("FPS")
                    .blurb("frame per seconds")
                    .minimum(Fraction::new(1, 1))
                    .maximum(Fraction::new(90, 1))
                    .default_value(Settings::default().fps)
                    .flags(ParamFlags::READWRITE)
                    .build(),
                glib::ParamSpecBoolean::builder("is-live")
                    .nick("Is Live")
                    .blurb("(Pseudo) live output")
                    .default_value(DEFAULT_IS_LIVE)
                    .mutable_ready()
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "fps" => {
                let x: Fraction = value.get().expect("type checkd upstream");
                gst::info!(CAT, imp: self, "set prop fps to {}", &x);
                let mut settings = self.settings.write().unwrap();
                settings.fps = x;
            }
            "is-live" => {
                let mut settings = self.settings.write().unwrap();
                let is_live = value.get().expect("type checked upstream");
                gst::info!(
                    CAT,
                    imp: self,
                    "Changing is-live from {} to {}",
                    settings.is_live,
                    is_live
                );
                settings.is_live = is_live;
            }

            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "fps" => {
                let settings = self.settings.read().unwrap();
                settings.fps.to_value()
            }
            "is-live" => {
                let settings = self.settings.read().unwrap();
                settings.is_live.to_value()
            }

            _ => unimplemented!(),
        }
    }
}

impl GstObjectImpl for KlvTestSrc {}

#[glib::object_subclass]
impl ObjectSubclass for KlvTestSrc {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::KlvTestSrc;
    type ParentType = gst_base::PushSrc;
}

impl BaseSrcImpl for KlvTestSrc {}

impl PushSrcImpl for KlvTestSrc {
    fn create(
        &self,
        _buffer: Option<&mut gst::BufferRef>,
    ) -> Result<gst_base::subclass::base_src::CreateSuccess, gst::FlowError> {
        gst::debug!(CAT, imp: self, "create PushSrcImpl");
        let count = self
            .count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (fps, is_live) = {
            let settings = self.settings.read().unwrap();
            (settings.fps.numer(), settings.is_live)
        };
        let meta = ExampleRsMetaParams::new(
            "KlvTestSrcLabel".to_string(),
            (count % i32::MAX as u64) as i32,
            ers_meta::TransformMode::Copy,
        );
        let records = encode_klv_params(&meta);
        let size = klv::encode_len(&records);
        let mut buffer = gst::Buffer::with_size(size).unwrap();
        {
            let mut bw = buffer.make_mut().map_writable().unwrap();
            // bw.copy_from_slice("testdata".as_bytes());
            klv::encode(&mut bw, &records).unwrap();
        }

        // fractionから設定可能にしたい
        let timeoffset = gst::ClockTime::SECOND / fps as u64;
        {
            let buffer = buffer.get_mut().unwrap();
            buffer.set_pts(Some(timeoffset * count));
            buffer.set_duration(Some(timeoffset));
            buffer.set_offset(count)
        }

        // リアルタイム処理の場合
        if is_live {
            let (clock, base_time) = match Option::zip(self.obj().clock(), self.obj().base_time()) {
                None => return Ok(CreateSuccess::NewBuffer(buffer)),
                Some(res) => res,
            };
            // シーク可能なセグメント生成
            let segment = self
                .obj()
                .segment()
                .downcast::<gst::format::Time>()
                .unwrap();
            let running_time = segment.to_running_time(buffer.pts().opt_add(buffer.duration()));

            // 待ち時間作成
            let wait_until = match running_time.opt_add(base_time) {
                Some(wait_until) => wait_until,
                None => return Ok(CreateSuccess::NewBuffer(buffer)),
            };

            // 実時間を待つ
            let mut clock_wait = self.clock_wait.lock().unwrap();
            // gst-plugin-rsのsinesrcでは存在するがデフォルトflushing=trueなので常に終了してしまうためコメントアウトしている
            // if clock_wait.flushing {
            //     gst::debug!(CAT, imp: self, "clock_wait Flushing");
            //     return Err(gst::FlowError::Flushing);
            // }

            // タイマー設定
            let id = clock.new_single_shot_id(wait_until);
            clock_wait.clock_id = Some(id.clone());
            drop(clock_wait);
            gst::log!(
                CAT,
                imp: self,
                "Waiting until {}, now {}",
                wait_until,
                clock.time().unwrap().display(),
            );
            let (res, jitter) = id.wait();
            gst::log!(CAT, imp: self, "Waited res {:?} jitter {}", res, jitter);
            self.clock_wait.lock().unwrap().clock_id.take();
            if res == Err(gst::ClockError::Unscheduled) {
                gst::debug!(CAT, imp: self, "Flushing");
                return Err(gst::FlowError::Flushing);
            }
        }

        gst::debug!(CAT, imp: self, "Produced buffer {:?}", buffer);

        Ok(CreateSuccess::NewBuffer(buffer))
    }
}
