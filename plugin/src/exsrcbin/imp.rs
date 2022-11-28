use gst::glib;
use gst::prelude::ElementExtManual;
use gst::prelude::GObjectExtManualGst;

use gst::subclass::prelude::*;
use gst::traits::ElementExt;

use once_cell::sync::Lazy;

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some("ExSrcBin"),
    )
});
#[derive(Default)]
pub struct ExSrcBin {
    // source: RwLock<GstElement>
}

impl ExSrcBin {
    // videosrcを設定する
    // videotestsrcとtypefindを繋ぐだけ
}

impl ElementImpl for ExSrcBin {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                CLASS_NAME,
                "Generic",
                "Test impl GstBin",
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
                gst::PadPresence::Sometimes,
                &caps,
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
        gst::info!(CAT, "Call change state {:?}", transition);
        // before play
        if transition == gst::StateChange::ReadyToPaused {
            // 子要素を自身に追加
            let videosrc = gst::ElementFactory::make("videotestsrc").build().unwrap();
            let addmeta = gst::ElementFactory::make("metatrans").build().unwrap();
            addmeta.set_property_from_str("op", "add");
            self.add_element(&videosrc).unwrap();
            self.add_element(&addmeta).unwrap();
            videosrc.link(&addmeta).unwrap();

            // binの前後のエレメントと繋ぐためにbinにGhostPadを作り最後の小要素のsrcをつなげる
            let pad = addmeta.static_pad("src").expect("Failed to get src pad");
            let ghost_pad = gst::GhostPad::with_target(Some("src"), &pad).unwrap();
            self.obj().add_pad(&ghost_pad).unwrap();
        }
        self.parent_change_state(transition)
        // after play!!
    }

    fn request_new_pad(
        &self,
        templ: &gst::PadTemplate,
        name: Option<&str>,
        caps: Option<&gst::Caps>,
    ) -> Option<gst::Pad> {
        gst::info!(CAT, "request_new_pad {:?} {:?}", name, caps);
        self.parent_request_new_pad(templ, name, caps)
    }

    fn release_pad(&self, pad: &gst::Pad) {
        gst::info!(CAT, "release_pad {:?}", pad);
        self.parent_release_pad(pad)
    }

    fn send_event(&self, event: gst::Event) -> bool {
        gst::info!(CAT, "send_event {:?}", event);
        self.parent_send_event(event)
    }
}
impl ObjectImpl for ExSrcBin {}
impl GstObjectImpl for ExSrcBin {}

#[glib::object_subclass]
impl ObjectSubclass for ExSrcBin {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::ExSrcBin;
    type ParentType = gst::Bin;
}

impl BinImpl for ExSrcBin {
    fn handle_message(&self, message: gst::Message) {
        gst::info!(CAT, "handle_message {:?}", message);
        self.parent_handle_message(message)
    }
}
