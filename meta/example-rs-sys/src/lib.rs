//! ExampleRsMeta binding for Rust
//!
//! ExampleRsMetaをRustで扱うためのバインディング実装

use gst::prelude::*;
mod imp;

#[link(name = "example_rs_meta")]
extern "C" {
    pub fn example_rs_meta_get_info() -> *const gst::ffi::GstMetaInfo;
    pub fn example_rs_meta_api_get_type() -> gst::glib::Type;
}

// Public Rust type for the custom meta.
#[repr(transparent)]
#[derive(Debug)]
pub struct ExampleRsMeta(imp::ExampleRsMeta);
pub use imp::ExampleRsMetaParams;
pub use imp::TransformMode;

// Metas must be Send+Sync.
unsafe impl Send for ExampleRsMeta {}
unsafe impl Sync for ExampleRsMeta {}

unsafe impl MetaAPI for ExampleRsMeta {
    type GstType = imp::ExampleRsMeta;

    fn meta_api() -> gst::glib::Type {
        unsafe { example_rs_meta_api_get_type() }
    }
}

impl ExampleRsMeta {
    // 別のバッファにデータをクローンするメソッド
    pub fn add(
        buffer: &mut gst::BufferRef,
        param: imp::ExampleRsMetaParams,
    ) -> gst::MetaRefMut<Self, gst::meta::Standalone> {
        unsafe {
            let mut params = std::mem::ManuallyDrop::new(param);
            let meta = gst::ffi::gst_buffer_add_meta(
                buffer.as_mut_ptr(),
                example_rs_meta_get_info(),
                &mut *params as *mut imp::ExampleRsMetaParams as gst::glib::ffi::gpointer,
            ) as *mut imp::ExampleRsMeta;

            Self::from_mut_ptr(buffer, meta)
        }
    }

    #[doc(alias = "get_label")]
    pub fn label(&self) -> &str {
        self.0.label.as_str()
    }

    #[doc(alias = "get_index")]
    pub fn index(&self) -> i32 {
        self.0.index
    }

    #[doc(alias = "get_mode")]
    pub fn mode(&self) -> imp::TransformMode {
        self.0.mode
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        imp::{ExampleRsMetaParams, TransformMode},
        ExampleRsMeta,
    };
    #[test]
    fn test_write_read() {
        const LABEL: &str = "testlabel";
        const INDEX: i32 = 12345;
        const MODE: TransformMode = TransformMode::Ignore;
        gst::init().unwrap();
        let mut buffer = gst::Buffer::with_size(1024).unwrap();
        {
            let buffer = buffer.make_mut();
            let params = ExampleRsMetaParams::new(LABEL.to_string(), INDEX, MODE);
            let _meta = ExampleRsMeta::add(buffer, params);
        }
        if let Some(meta) = buffer.meta::<ExampleRsMeta>() {
            assert_eq!(meta.label(), LABEL);
            assert_eq!(meta.index(), INDEX);
            assert_eq!(meta.mode(), MODE);
        }
    }
}
