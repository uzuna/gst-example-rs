use std::sync::Mutex;

use gst::glib;
use gst::prelude::ParamSpecBuilderExt;
use gst::prelude::ToValue;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::BaseTransformImpl;
use once_cell::sync::Lazy;
use std::convert::AsRef;

use super::CLASS_NAME;
use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some("TestTrans Element"),
    )
});

// BaseTransformerのバッファコピーモード選択
// コピー方法でメタデータなどがどのように変化するのかを比較可能にする
// glib::Enum定義が必要なのはEnumをGStreamerのGtypeとして登録させる必要があるから
// なのでenum_typeで一意の名前をつける必要があり
// enum_valueを使ってgst-inspect向けに詳細情報を付与する
#[derive(Debug, Default, PartialEq, Clone, Copy, glib::Enum)]
#[enum_type(name = "TestTransCopyMode")]
enum CopyMode {
    #[enum_value(name = "Timestamp: copy timestamp only")]
    Timestamp = 0,
    #[default]
    #[enum_value(name = "Meta: copy timestamp and meta")]
    Meta = 1,
    #[enum_value(name = "MetaOnly: copy meta only")]
    MetaOnly = 2,
    #[enum_value(name = "MetaDeep: copy meta with deepflag")]
    MetaDeep = 3,
    #[enum_value(name = "Memory: copy memory")]
    Memory = 4,
    #[enum_value(name = "All: copy all")]
    All = 5,
}

impl CopyMode {
    fn buffer_copy_flag(&self) -> gst::BufferCopyFlags {
        use gst::BufferCopyFlags;
        match self {
            CopyMode::Timestamp => BufferCopyFlags::TIMESTAMPS,
            CopyMode::MetaOnly => BufferCopyFlags::META,
            CopyMode::Meta => BufferCopyFlags::TIMESTAMPS | BufferCopyFlags::META,
            CopyMode::MetaDeep => {
                BufferCopyFlags::TIMESTAMPS | BufferCopyFlags::META | BufferCopyFlags::DEEP
            }
            CopyMode::Memory => BufferCopyFlags::MEMORY,
            CopyMode::All => BufferCopyFlags::all(),
        }
    }
}

#[derive(Debug, Default)]
struct Settings {
    copy_mode: CopyMode,
}

impl Settings {
    fn set_copy_mode(&mut self, v: CopyMode) {
        self.copy_mode = v
    }
}

#[derive(Default)]
pub struct TestTrans {
    settings: Mutex<Settings>,
}

impl TestTrans {}

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
    // Metadata for the property
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                gst::glib::ParamSpecEnum::builder::<CopyMode>("copymode", CopyMode::default())
                    .nick("CopyMode")
                    .blurb("select copy mode")
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    // gstreamerの起動時プロパティ設定
    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "copymode" => {
                let x = value.get::<CopyMode>().expect("type checkd upstream");
                gst::info!(CAT, imp: self, "set prop copymode to {:?}", x);
                let mut settings = self.settings.lock().unwrap();
                settings.set_copy_mode(x);
            }
            _ => unimplemented!(),
        }
    }

    // propertiesに含まれるParamSpecについて実装が必要
    // 実装や対象のpropertyがない場合はinimplementsに到達してgst-inspectがabortする
    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "copymode" => {
                let settings = self.settings.lock().unwrap();
                settings.copy_mode.to_value()
            }
            _ => unimplemented!(),
        }
    }
}

impl GstObjectImpl for TestTrans {}

#[glib::object_subclass]
impl ObjectSubclass for TestTrans {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::TestTrans;
    type ParentType = gst_base::BaseTransform;
}

impl BaseTransformImpl for TestTrans {
    // Bufferの使い方についてヒントを出す
    // InPlace=InBufferを直接書き換えるか否か、両方か
    const MODE: gst_base::subclass::BaseTransformMode = gst_base::subclass::BaseTransformMode::Both;
    // 同じCapsの場合にパススルーするフラグ設定
    // When AlwaysInPlace && true must impl `transform_ip_passthrough`
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    // Inplaceの場合にPassthroughするフラグ設定
    // When AlwaysInPlace && PASSTHROUGH_ON_SAME_CAPS == false && true
    // must impl `transform_ip`
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = true;

    fn transform(
        &self,
        inbuf: &gst::Buffer,
        outbuf: &mut gst::BufferRef,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        let copy_mode = {
            let settings = self.settings.lock().unwrap();
            settings.copy_mode
        };

        use CopyMode::*;
        match copy_mode {
            Timestamp | Meta | MetaOnly | MetaDeep => {
                // バッファを確保済みの領域にコピーをする
                {
                    let mut bw = outbuf.map_writable().unwrap();
                    let br = inbuf.map_readable().unwrap();
                    bw.copy_from_slice(br.as_slice());
                }
            }
            Memory | All => {
                outbuf.remove_all_memory();
            }
        }

        // copy_intoは基本的にメタデータのコピーに使う
        // 通常は TIMESTAMPS | META で良い
        // FLAGSはSTBufferの制御のためでMEMORYの場合には使うのかも?
        // TODO: DEEPはMETAの情報を揮発させないために必要?
        // MEMORYを指定することでバッファもコピーできるが確保済み領域に追加されるので
        // 事前にoutbuf.remove_all_memoryで全てのメモリを破棄しなければ期待するデータにならない
        // この挙動はcopy_intoの前後でsize()を比較で見ることが出来る
        inbuf
            .copy_into(outbuf, copy_mode.buffer_copy_flag(), 0, None)
            .map_err(|_| gst::FlowError::Error)?;
        gst::trace!(CAT, imp: self, "transform");
        Ok(gst::FlowSuccess::Ok)
    }

    fn transform_ip(&self, _buf: &mut gst::BufferRef) -> Result<gst::FlowSuccess, gst::FlowError> {
        gst::trace!(CAT, imp: self, "transform_ip");
        Ok(gst::FlowSuccess::Ok)
    }

    fn transform_ip_passthrough(
        &self,
        _buf: &gst::Buffer,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        gst::trace!(CAT, imp: self, "transform_ip_passthrough");
        Ok(gst::FlowSuccess::Ok)
    }
}
