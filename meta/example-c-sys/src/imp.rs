//! ExampleCMeta structs
//!
//! ExampleCMetaと同じ型を定義する

#[repr(C)]
#[derive(Debug, Default)]
pub struct ExampleCMetaParams {
    pub count: i64,
    pub num: f32,
}

impl ExampleCMetaParams {
    pub fn new(count: i64, num: f32) -> Self {
        Self { count, num }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ExampleCMeta {
    parent: gst::ffi::GstMeta,
    pub count: i64,
    pub num: f32,
}
