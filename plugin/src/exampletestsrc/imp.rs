use gst::glib;
use gst::glib::ParamFlags;
use gst::prelude::ElementExtManual;
use gst::prelude::GObjectExtManualGst;
use gst::prelude::ParamSpecBuilderExt;
use gst::prelude::ToValue;
use gst::subclass::prelude::*;
use gst::traits::ElementExt;
use gst::Fraction;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some(CLASS_NAME),
    )
});

#[derive(Debug)]
struct Settings {
    fps: Fraction,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: Fraction::new(15, 1),
        }
    }
}

#[derive(Default)]
pub struct ExampleTestSrc {
    settings: RwLock<Settings>,
}

impl ExampleTestSrc {
    /// 自身に子要素を自身に追加して再生可能にする
    fn setup_source(&self) -> Result<(), glib::BoolError> {
        let videosrc = gst::ElementFactory::make("videotestsrc").build()?;
        let addmeta = gst::ElementFactory::make("metatrans").build()?;
        let scale = gst::ElementFactory::make("videoscale").build()?;

        addmeta.set_property_from_str("op", "add");

        let caps = {
            let settings = self.settings.read();
            gst::Caps::builder("video/x-raw")
                .field("width", 600)
                .field("height", 400)
                .field("framerate", settings.fps)
                .build()
        };
        self.add_element(&videosrc).unwrap();
        self.add_element(&addmeta).unwrap();
        self.add_element(&scale).unwrap();
        videosrc.link(&addmeta).unwrap();
        addmeta.link_filtered(&scale, &caps)?;

        // binの前後のエレメントと繋ぐためにbinにGhostPadを作り最後の小要素のsrcをつなげる
        let pad = scale.static_pad("src").expect("Failed to get src pad");
        let ghost_pad = gst::GhostPad::with_target(Some("src"), &pad)?;
        self.obj().add_pad(&ghost_pad)?;
        Ok(())
    }
}

impl ElementImpl for ExampleTestSrc {
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
            self.setup_source().unwrap();
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
impl ObjectImpl for ExampleTestSrc {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![gst::param_spec::ParamSpecFraction::builder("fps")
                .nick("FPS")
                .blurb("frame per seconds")
                .minimum(Fraction::new(1, 1))
                .maximum(Fraction::new(90, 1))
                .default_value(Settings::default().fps)
                .flags(ParamFlags::READWRITE)
                .build()]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "fps" => {
                let x: Fraction = value.get().expect("type checkd upstream");
                gst::info!(CAT, imp: self, "set prop fps to {}", &x);
                let mut settings = self.settings.write();
                settings.fps = x;
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "fps" => {
                let settings = self.settings.read();
                settings.fps.to_value()
            }
            _ => unimplemented!(),
        }
    }
}
impl GstObjectImpl for ExampleTestSrc {}

#[glib::object_subclass]
impl ObjectSubclass for ExampleTestSrc {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::ExampleTestSrc;
    type ParentType = gst::Bin;
}

impl BinImpl for ExampleTestSrc {
    fn handle_message(&self, message: gst::Message) {
        gst::info!(CAT, "handle_message {:?}", message);
        self.parent_handle_message(message)
    }
}
