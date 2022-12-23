use gst::glib;
use gst::prelude::*;

const ELEMENT_NAME: &str = "simpletrans";
const CLASS_NAME: &str = "SimpleTrans";

mod imp;

gst::glib::wrapper! {
    // 継承するクラスをここで宣言する
    // 少なくとも gst::Element, gst::Object が必要でObjectSubclassでParentTypeに指定するクラスをここに書く
    // 今回はgst_base::BaseTransform
    pub struct SimpleTrans(ObjectSubclass<imp::SimpleTrans>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

// gstreamerにこのエレメントを登録する
// gstremaerは起動時にpluginのあるディレクトリからダイナミックリンクライブラリを読み
// エレメント以外にもメタデータやgstreamer内で一意になるように情報を割り当てるため、
// 何らかの構造体を使い時には登録が必要
pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        ELEMENT_NAME,
        gst::Rank::None,
        SimpleTrans::static_type(),
    )
}
