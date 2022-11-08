use gst::glib;
use gst::prelude::ElementClassExt;
use gst::prelude::PadExtManual;
use gst::subclass::prelude::*;
use gst::traits::ElementExt;
use once_cell::sync::Lazy;

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some("TestTrans Element"),
    )
});

pub struct TestTrans {
    srcpad: gst::Pad,
    sinkpad: gst::Pad,
}

impl TestTrans {
    fn sink_chain(
        &self,
        pad: &gst::Pad,
        buffer: gst::Buffer,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        gst::log!(CAT, obj: pad, "Handling buffer {:?}", buffer);
        self.srcpad.push(buffer)
    }
    fn sink_event(&self, pad: &gst::Pad, event: gst::Event) -> bool {
        gst::log!(CAT, obj: pad, "Handling event {:?}", event);
        self.srcpad.push_event(event)
    }
    fn sink_query(&self, pad: &gst::Pad, query: &mut gst::QueryRef) -> bool {
        gst::log!(CAT, obj: pad, "Handling query {:?}", query);
        self.srcpad.peer_query(query)
    }
    fn src_event(&self, pad: &gst::Pad, event: gst::Event) -> bool {
        gst::log!(CAT, obj: pad, "Handling event {:?}", event);
        self.sinkpad.push_event(event)
    }
    fn src_query(&self, pad: &gst::Pad, query: &mut gst::QueryRef) -> bool {
        gst::log!(CAT, obj: pad, "Handling query {:?}", query);
        self.sinkpad.peer_query(query)
    }
}

impl ElementImpl for TestTrans {
    // エレメントの仕様について記述する
    // gst-inspect-1.0で表示される情報でgst::Registryで登録されメモリ上に保持される
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                CLASS_NAME,
                "Generic",
                "To see the difference in BaseTransform behavior",
                "FUJINAKA Fumiya <uzuna.kf@gmail.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    // sink, srcのpad templateを作る
    // 前後のエレメントとのネゴシエーションに使う
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

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }

    // パイプライン状態が変わる時に呼ばれる
    // 多くの場合はリソースの確保や開放等を行う
    fn change_state(
        &self,
        transition: gst::StateChange,
    ) -> Result<gst::StateChangeSuccess, gst::StateChangeError> {
        gst::trace!(CAT, imp: self, "Changing state {:?}", transition);
        self.parent_change_state(transition)
    }
}

impl ObjectImpl for TestTrans {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.add_pad(&self.sinkpad).unwrap();
        obj.add_pad(&self.srcpad).unwrap();
    }
}

impl GstObjectImpl for TestTrans {}

#[glib::object_subclass]
impl ObjectSubclass for TestTrans {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::TestTrans;
    type ParentType = gst::Element;

    fn with_class(klass: &Self::Class) -> Self {
        let templ = klass.pad_template("sink").unwrap();
        let sinkpad = gst::Pad::builder_with_template(&templ, Some("sink"))
            .chain_function(|pad, parent, buffer| {
                TestTrans::catch_panic_pad_function(
                    parent,
                    || Err(gst::FlowError::Error),
                    |identity| identity.sink_chain(pad, buffer),
                )
            })
            .event_function(|pad, parent, event| {
                TestTrans::catch_panic_pad_function(
                    parent,
                    || false,
                    |identity| identity.sink_event(pad, event),
                )
            })
            .query_function(|pad, parent, query| {
                TestTrans::catch_panic_pad_function(
                    parent,
                    || false,
                    |identity| identity.sink_query(pad, query),
                )
            })
            .build();

        let templ = klass.pad_template("src").unwrap();
        let srcpad = gst::Pad::builder_with_template(&templ, Some("src"))
            .event_function(|pad, parent, event| {
                TestTrans::catch_panic_pad_function(
                    parent,
                    || false,
                    |identity| identity.src_event(pad, event),
                )
            })
            .query_function(|pad, parent, query| {
                TestTrans::catch_panic_pad_function(
                    parent,
                    || false,
                    |identity| identity.src_query(pad, query),
                )
            })
            .build();
        Self { srcpad, sinkpad }
    }
}
