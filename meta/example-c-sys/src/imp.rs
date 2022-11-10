//! ExampleCMeta structs
//!
//! ExampleCMetaと同じ型を定義する

#[derive(Debug, Default)]
pub struct ExampleCMetaParams {
    pub count: i32,
    pub num: f32,
}

impl ExampleCMetaParams {
    pub fn new(count: i32, num: f32) -> Self {
        Self { count, num }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ExampleCMeta {
    parent: gst::ffi::GstMeta,
    pub count: i32,
    pub num: f32,
}
