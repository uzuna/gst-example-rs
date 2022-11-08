use gst::glib;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::BaseTransformImpl;
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

#[derive(Default)]
pub struct TestTrans {}

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

impl ObjectImpl for TestTrans {}

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
        // バッファを確保済みの領域にコピーをする
        {
            let mut bw = outbuf.map_writable().unwrap();
            let br = inbuf.map_readable().unwrap();
            bw.copy_from_slice(br.as_slice());
        }

        // copy_intoは基本的にメタデータのコピーに使う
        // 通常は TIMESTAMPS | META で良い
        // FLAGSはSTBufferの制御のためでMEMORYの場合には使うのかも?
        // TODO: DEEPはMETAの情報を揮発させないために必要?
        // MEMORYを指定することでバッファもコピーできるが確保済み領域に追加されるので
        // 事前にoutbuf.remove_all_memoryで全てのメモリを破棄しなければ期待するデータにならない
        // この挙動はcopy_intoの前後でsize()を比較で見ることが出来る
        inbuf
            .copy_into(
                outbuf,
                gst::BufferCopyFlags::TIMESTAMPS
                    | gst::BufferCopyFlags::META
                    | gst::BufferCopyFlags::DEEP,
                0,
                None,
            )
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
