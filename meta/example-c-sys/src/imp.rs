//! ExampleCMeta structs
//!
//! ExampleCMetaと同じ型を定義する

#[repr(C)]
#[derive(Debug)]
pub struct ExampleCMetaParams {
    pub label: gst::glib::GString,
    pub count: i64,
    pub num: f32,
}

impl ExampleCMetaParams {
    pub fn new(label: String, count: i64, num: f32) -> Self {
        Self {
            label: label.into(),
            count,
            num,
        }
    }
}
// TODO double free issue
// impl Drop for ExampleCMetaParams {
//     fn drop(&mut self) {
//         println!("drop on ExampleCMetaParams");
//     }
// }

#[repr(C)]
#[derive(Debug)]
pub struct ExampleCMeta {
    parent: gst::ffi::GstMeta,
    pub label: gst::glib::GString,
    pub count: i64,
    pub num: f32,
}

// impl Drop for ExampleCMeta {
//     fn drop(&mut self) {
//         println!("drop on ExampleCMeta");
//     }
// }
