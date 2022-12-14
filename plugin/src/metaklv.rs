//! Ecample metaklv impl
use ers_meta::{ExampleRsMeta, ExampleRsMetaParams};
use gst::Caps;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "gstexamplers0000")]
pub struct ExampleDataset {
    #[serde(rename = "2")]
    index: i32,
    #[serde(rename = "3")]
    mode: u32,
    #[serde(rename = "16")]
    label: String,
}

impl From<&ExampleRsMeta> for ExampleDataset {
    fn from(meta: &ExampleRsMeta) -> Self {
        Self {
            index: meta.index(),
            mode: meta.mode() as u32,
            label: meta.label().to_string(),
        }
    }
}

impl From<&ExampleRsMetaParams> for ExampleDataset {
    fn from(params: &ExampleRsMetaParams) -> Self {
        Self {
            index: params.index,
            mode: params.mode as u32,
            label: params.label.to_string(),
        }
    }
}

impl Into<ExampleRsMetaParams> for ExampleDataset {
    fn into(self) -> ExampleRsMetaParams {
        ExampleRsMetaParams {
            index: self.index,
            mode: self.mode.into(),
            label: self.label,
        }
    }
}

pub static KLV_CAPS: Lazy<Caps> = Lazy::new(|| {
    gst::Caps::builder("meta/x-klv")
        .field("parsed", true)
        .build()
});
