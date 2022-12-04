use std::sync::atomic::AtomicI32;
use std::sync::Mutex;

use ers_meta::ExampleRsMetaParams;
use gst::glib::ParamFlags;
use gst::prelude::{
    ClockExtManual, Displayable, ElementExtManual, GObjectExtManualGst, OptionAdd,
    ParamSpecBuilderExt, ToValue,
};
use gst::subclass::prelude::*;
use gst::traits::{ClockExt, ElementExt};
use gst::{glib, Caps};
use gst::{ClockTime, Fraction};

use gst_base::prelude::BaseSrcExtManual;
use gst_base::subclass::base_src::CreateSuccess;
use gst_base::subclass::prelude::{BaseSrcImpl, BaseSrcImplExt, PushSrcImpl};
use gst_base::traits::BaseSrcExt;
use gst_base::PushSrc;
use once_cell::sync::Lazy;

use crate::metatrans::metaklv::encode_klv_params;

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

struct Settings {
    fps: Fraction,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: Fraction::new(30, 1),
        }
    }
}

struct ClockWait {
    clock_id: Option<gst::SingleShotClockId>,
    flushing: bool,
}

impl Default for ClockWait {
    fn default() -> ClockWait {
        ClockWait {
            clock_id: None,
            flushing: true,
        }
    }
}

#[derive(Default)]
pub struct KlvTestSrc {
    count: AtomicI32,
    clock_wait: Mutex<ClockWait>,
    settings: Mutex<Settings>,
}

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
        self.obj().set_live(true);
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
        obj.set_live(true);
        obj.set_format(gst::Format::Time);
    }
}

impl GstObjectImpl for KlvTestSrc {}

#[glib::object_subclass]
impl ObjectSubclass for KlvTestSrc {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::KlvTestSrc;
    type ParentType = gst_base::PushSrc;
}

impl BaseSrcImpl for KlvTestSrc {
    fn set_caps(&self, caps: &gst::Caps) -> Result<(), gst::LoggableError> {
        gst::debug!(CAT, imp: self, "Configuring for caps {}", caps);

        let _ = self
            .obj()
            .post_message(gst::message::Latency::builder().src(&*self.obj()).build());

        Ok(())
    }
}

impl PushSrcImpl for KlvTestSrc {
    fn create(
        &self,
        _buffer: Option<&mut gst::BufferRef>,
    ) -> Result<gst_base::subclass::base_src::CreateSuccess, gst::FlowError> {
        gst::debug!(CAT, imp: self, "create PushSrcImpl");
        let count = self
            .count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let meta = ExampleRsMetaParams::new(
            "KlvTestSrcLabel".to_string(),
            count,
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

        // let (clock, base_time) = match Option::zip(self.obj().clock(), self.obj().base_time()) {
        //     None => return Ok(CreateSuccess::NewBuffer(buffer)),
        //     Some(res) => res,
        // };
        // 後段がsync=trueならこれが意味を持つ
        // fractionから設定可能にしたい
        {
            let buffer = buffer.get_mut().unwrap();
            buffer.set_pts(Some(gst::ClockTime::SECOND * count as u64));
            buffer.set_dts(None);
            buffer.set_duration(Some(gst::ClockTime::SECOND));
        }
        // let segment = self
        //     .obj()
        //     .segment()
        //     .downcast::<gst::format::Time>()
        //     .unwrap();
        // let running_time = segment.to_running_time(buffer.pts().opt_add(buffer.duration()));

        // let wait_until = match running_time.opt_add(base_time) {
        //     Some(wait_until) => wait_until,
        //     None => return Ok(CreateSuccess::NewBuffer(buffer)),
        // };
        // let mut clock_wait = self.clock_wait.lock().unwrap();
        // if clock_wait.flushing {
        //     gst::debug!(CAT, imp: self, "Flushing");
        //     return Err(gst::FlowError::Flushing);
        // }

        // let id = clock.new_single_shot_id(wait_until);
        // clock_wait.clock_id = Some(id.clone());
        // drop(clock_wait);

        // gst::log!(
        //     CAT,
        //     imp: self,
        //     "Waiting until {}, now {}",
        //     wait_until,
        //     clock.time().unwrap().display(),
        // );
        // let (res, jitter) = id.wait();
        // gst::log!(CAT, imp: self, "Waited res {:?} jitter {}", res, jitter);
        // self.clock_wait.lock().unwrap().clock_id.take();

        // // If the clock ID was unscheduled, unlock() was called
        // // and we should return Flushing immediately.
        // if res == Err(gst::ClockError::Unscheduled) {
        //     gst::debug!(CAT, imp: self, "Flushing");
        //     return Err(gst::FlowError::Flushing);
        // }

        // gst::debug!(CAT, imp: self, "Produced buffer {:?}", buffer);

        Ok(CreateSuccess::NewBuffer(buffer))
    }
}
