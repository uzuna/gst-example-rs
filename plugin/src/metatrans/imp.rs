use std::sync::atomic::AtomicI32;

use ec_meta::{ExampleCMeta, ExampleCMetaParams};
use gst::traits::GstObjectExt;
use parking_lot::RwLock;

use ers_meta::{ExampleRsMeta, ExampleRsMetaParams, TransformMode};
use gst::glib::{self, ParamFlags};
use gst::prelude::{ParamSpecBuilderExt, ToValue};
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::BaseTransformImpl;
use once_cell::sync::Lazy;
use strum::{AsRefStr, EnumString};

use crate::metatrans::CLASS_NAME;

use super::ELEMENT_NAME;

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some("MetaTrans Element"),
    )
});

/// metadata operation mode
#[derive(AsRefStr, Debug, EnumString, PartialEq, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
enum TransformMethod {
    // no action in transform
    Ignore,
    // copy to dest buffer
    Copy,
}

impl Default for TransformMethod {
    fn default() -> Self {
        Self::Copy
    }
}

#[allow(clippy::from_over_into)]
impl Into<TransformMode> for TransformMethod {
    fn into(self) -> TransformMode {
        match self {
            TransformMethod::Ignore => TransformMode::Ignore,
            TransformMethod::Copy => TransformMode::Copy,
        }
    }
}

/// metadata operation mode
#[derive(AsRefStr, Debug, EnumString, PartialEq, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
enum OperationMode {
    // show metadata
    Show,
    // add metadata
    Add,
    // remove metadata
    Remove,
}

impl Default for OperationMode {
    fn default() -> Self {
        Self::Show
    }
}

/// 使うメタデータの種類を切り替える
#[derive(AsRefStr, Debug, EnumString, PartialEq, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
enum MetaType {
    // Rust実装
    Rs,
    // C実装
    C,
}

impl Default for MetaType {
    fn default() -> Self {
        Self::Rs
    }
}

#[derive(Debug, Default)]
struct Settings {
    op_mode: OperationMode,
    transform_meta: TransformMethod,
    meta_type: MetaType,
}

impl Settings {
    fn set_op_mode(&mut self, new_mode: &str) -> Result<(), String> {
        match OperationMode::try_from(new_mode) {
            Ok(mode) => {
                self.op_mode = mode;
                Ok(())
            }
            _ => Err(format!("invalid copy mode {}", new_mode)),
        }
    }
    fn set_transform_method(&mut self, new_mode: &str) -> Result<(), String> {
        match TransformMethod::try_from(new_mode) {
            Ok(meta) => {
                self.transform_meta = meta;
                Ok(())
            }
            _ => Err(format!("invalid copy mode {}", new_mode)),
        }
    }
    fn set_meta_type(&mut self, new_mode: &str) -> Result<(), String> {
        match MetaType::try_from(new_mode) {
            Ok(meta) => {
                self.meta_type = meta;
                Ok(())
            }
            _ => Err(format!("invalid meta_type {}", new_mode)),
        }
    }
}

#[derive(Default)]
pub struct MetaTrans {
    settings: RwLock<Settings>,
    count: AtomicI32,
}

impl ElementImpl for MetaTrans {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                CLASS_NAME,
                "Generic",
                "set, display, remove example-metadata",
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

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }

    fn change_state(
        &self,
        transition: gst::StateChange,
    ) -> Result<gst::StateChangeSuccess, gst::StateChangeError> {
        gst::trace!(CAT, imp: self, "Changing state {:?}", transition);
        self.parent_change_state(transition)
    }
}

impl ObjectImpl for MetaTrans {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                gst::glib::ParamSpecString::builder("op")
                    .nick("Operation")
                    .blurb("select operation mode")
                    .default_value(OperationMode::default().as_ref())
                    .flags(ParamFlags::READWRITE)
                    .build(),
                gst::glib::ParamSpecString::builder("tmethod")
                    .nick("Transform")
                    .blurb("select transform method")
                    .default_value(TransformMethod::default().as_ref())
                    .flags(ParamFlags::READWRITE)
                    .build(),
                gst::glib::ParamSpecString::builder("mtype")
                    .nick("Metatype")
                    .blurb("select metadata type")
                    .default_value(MetaType::default().as_ref())
                    .flags(ParamFlags::READWRITE)
                    .build(),
            ]
        });

        PROPERTIES.as_ref()
    }

    // gstreamerの起動時プロパティ設定
    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "op" => {
                let x: String = value.get().expect("type checked upstream");
                gst::info!(CAT, imp: self, "set prop op to {}", &x);
                let mut settings = self.settings.write();
                settings.set_op_mode(&x).expect("set op mode");
            }
            "tmethod" => {
                let x: String = value.get().expect("type checked upstream");
                gst::info!(CAT, imp: self, "set tmethod to {}", &x);
                let mut settings = self.settings.write();
                settings
                    .set_transform_method(&x)
                    .expect("set transform method");
            }
            "mtype" => {
                let x: String = value.get().expect("type checked upstream");
                gst::info!(CAT, imp: self, "set metatype to {}", &x);
                let mut settings = self.settings.write();
                settings.set_meta_type(&x).expect("set metatype");
            }
            _ => unimplemented!(),
        }
    }

    // propertiesに含まれるParamSpecについて実装が必要
    // 実装や対象のpropertyがない場合はinimplementsに到達してgst-inspectがabortする
    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "op" => {
                let settings = self.settings.read();
                settings.op_mode.as_ref().to_value()
            }
            "tmethod" => {
                let settings = self.settings.read();
                settings.transform_meta.as_ref().to_value()
            }
            "mtype" => {
                let settings = self.settings.read();
                settings.meta_type.as_ref().to_value()
            }
            _ => unimplemented!(),
        }
    }
}

impl GstObjectImpl for MetaTrans {}

#[glib::object_subclass]
impl ObjectSubclass for MetaTrans {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::MetaTrans;
    type ParentType = gst_base::BaseTransform;
}

impl BaseTransformImpl for MetaTrans {
    const MODE: gst_base::subclass::BaseTransformMode =
        gst_base::subclass::BaseTransformMode::AlwaysInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = true;

    fn transform_ip(
        &self,
        buffer: &mut gst::BufferRef,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        let (op_mode, transform_meta, meta_type) = {
            let settings = self.settings.read();
            (
                settings.op_mode,
                settings.transform_meta,
                settings.meta_type,
            )
        };
        match op_mode {
            OperationMode::Show => match meta_type {
                MetaType::Rs => {
                    if let Some(meta) = buffer.meta::<ExampleRsMeta>() {
                        gst::trace!(
                            CAT,
                            imp: self,
                            "found Rs meta ({:?}): {} {} {:?}",
                            buffer.pts(),
                            &meta.label(),
                            &meta.index(),
                            &meta.mode(),
                        );
                    } else {
                        gst::trace!(CAT, imp: self, "has not Rs metadata");
                    }
                }
                MetaType::C => {
                    if let Some(meta) = buffer.meta::<ExampleCMeta>() {
                        gst::trace!(
                            CAT,
                            imp: self,
                            "found C meta ({:?}): {} {} {:?}",
                            buffer.pts(),
                            &meta.label().as_str(),
                            &meta.count(),
                            &meta.num(),
                        );
                    } else {
                        gst::trace!(CAT, imp: self, "has not C metadata");
                    }
                }
            },
            OperationMode::Add => {
                // このプラグイン内では競合操作がないのでRelaxed
                let count = self
                    .count
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let msg_type = match meta_type {
                    MetaType::Rs => {
                        let param = ExampleRsMetaParams::new(
                            self.instance().name().to_string(),
                            count,
                            transform_meta.into(),
                        );
                        ers_meta::ExampleRsMeta::add(buffer, param);
                        "Rs Meta"
                    }
                    MetaType::C => {
                        let param = ExampleCMetaParams::new(
                            self.instance().name().to_string(),
                            count.into(),
                            count as f32 / 10.0,
                        );
                        ec_meta::ExampleCMeta::add(buffer, param);
                        "C Meta"
                    }
                };

                gst::trace!(
                    CAT,
                    imp: self,
                    "set meta {} ({:?}): {} {:?}",
                    msg_type,
                    buffer.pts(),
                    self.instance().name(),
                    count,
                );
            }
            OperationMode::Remove => {
                // TODO 調査
                // ユニットテストでは削除できているがgst-launchでは削除できていない

                match meta_type {
                    MetaType::Rs => {
                        if let Some(param) = ers_meta::ExampleRsMeta::remove(buffer) {
                            gst::trace!(
                                CAT,
                                imp: self,
                                "remove Rs meta ({:?}): {} {} {:?}",
                                buffer.pts(),
                                param.label,
                                param.index,
                                param.mode,
                            );
                        }
                    }
                    MetaType::C => {
                        ec_meta::ExampleCMeta::remove(buffer);
                        gst::trace!(CAT, imp: self, "remove C meta ({:?})", buffer.pts(),);
                    }
                }
            }
        }
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
