use gst::glib;
use gst::prelude::{ElementClassExt, PadExtManual};
use gst::subclass::prelude::{
    ElementImpl, ElementImplExt, GstObjectImpl, ObjectImpl, ObjectImplExt, ObjectSubclass,
    ObjectSubclassExt,
};
use gst::traits::ElementExt;
use gst::Caps;
use once_cell::sync::Lazy;

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
    // klvsrcpad: Mutex<Option<gst::Pad>>,
}

impl MetaDemux {
    fn sink_chain(
        &self,
        pad: &gst::Pad,
        buffer: gst::Buffer,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        gst::trace!(CAT, obj: pad, "Handling buffer {:?}", buffer);
        self.srcpad.push(buffer)
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
        Self { sinkpad, srcpad }
    }
}
