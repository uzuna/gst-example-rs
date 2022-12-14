//! ExampleRsMeta structs
//!
//! ExampleRsMetaと同じ型を定義する
//! TODO 型定義だけ共有できる方法を考えたい

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformMode {
    Ignore = 0,
    Copy = 1,
}

impl Default for TransformMode {
    fn default() -> Self {
        Self::Copy
    }
}

impl From<u32> for TransformMode {
    fn from(x: u32) -> Self {
        match x {
            0 => Self::Ignore,
            _ => Self::Copy,
        }
    }
}

#[derive(Debug, Default)]
pub struct ExampleRsMetaParams {
    pub label: String,
    pub index: i32,
    pub mode: TransformMode,
}

impl ExampleRsMetaParams {
    pub fn new(label: String, index: i32, mode: TransformMode) -> Self {
        Self { label, index, mode }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ExampleRsMeta {
    parent: gst::ffi::GstMeta,
    pub label: String,
    pub index: i32,
    pub mode: TransformMode,
}
