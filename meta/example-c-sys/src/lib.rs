//! ExampleCMeta binding for Rust
//!
//! ExampleCMetaをRustで扱うためのバインディング実装

use gst::{ffi::GstBuffer, prelude::*};
mod imp;

#[link(name = "example_c_meta")]
extern "C" {
    pub fn example_c_meta_get_info() -> *const gst::ffi::GstMetaInfo;
    pub fn example_c_meta_api_get_type() -> gst::glib::Type;
    pub fn buffer_add_example_c_meta(
        buffer: *mut GstBuffer,
        count: i64,
        num: f32,
    ) -> *mut imp::ExampleCMeta;
    //
    pub fn buffer_add_param_example_c_meta(
        buffer: *mut GstBuffer,
        param: &ExampleCMetaParams,
    ) -> *mut imp::ExampleCMeta;
}

// Public Rust type for the custom meta.
#[repr(transparent)]
#[derive(Debug)]
pub struct ExampleCMeta(imp::ExampleCMeta);
pub use imp::ExampleCMetaParams;

// Metas must be Send+Sync.
unsafe impl Send for ExampleCMeta {}
unsafe impl Sync for ExampleCMeta {}

unsafe impl MetaAPI for ExampleCMeta {
    type GstType = imp::ExampleCMeta;

    fn meta_api() -> gst::glib::Type {
        unsafe { example_c_meta_api_get_type() }
    }
}

impl ExampleCMeta {
    // 別のバッファにデータをクローンするメソッド
    pub fn add(
        buffer: &mut gst::BufferRef,
        param: imp::ExampleCMetaParams,
    ) -> gst::MetaRefMut<Self, gst::meta::Standalone> {
        unsafe {
            // OK
            // direct call gst_buffer_add_meta
            // use null param and write fields
            // let mut params = std::mem::ManuallyDrop::new(param);
            // let meta = gst::ffi::gst_buffer_add_meta(
            //     buffer.as_mut_ptr(),
            //     example_c_meta_get_info(),
            //     &mut *params as *mut ExampleCMetaParams as gst::glib::ffi::gpointer ,
            // ) as *mut imp::ExampleCMeta;

            // OK
            // use separated funcion
            // let meta = buffer_add_example_c_meta(
            //     buffer.as_mut_ptr(),
            //     param.count,
            //     param.num
            // );

            // OK
            // use struct param and ewassing in C++
            let meta = buffer_add_param_example_c_meta(buffer.as_mut_ptr(), &param);

            Self::from_mut_ptr(buffer, meta)
        }
    }

    pub fn remove(buffer: &mut gst::BufferRef) -> Option<imp::ExampleCMetaParams> {
        if let Some(meta) = buffer.meta_mut::<Self>() {
            let params = imp::ExampleCMetaParams {
                count: meta.count(),
                num: meta.num(),
            };
            meta.remove().unwrap();
            Some(params)
        } else {
            None
        }
    }

    #[doc(alias = "get_count")]
    pub fn count(&self) -> i64 {
        self.0.count
    }

    #[doc(alias = "get_num")]
    pub fn num(&self) -> f32 {
        self.0.num
    }
}

#[cfg(test)]
mod tests {
    use crate::{imp::ExampleCMetaParams, ExampleCMeta};
    #[test]
    fn test_write_read() {
        const COUNT: i64 = 12345;
        const NUM: f32 = 1.2345;
        gst::init().unwrap();
        let mut buffer = gst::Buffer::with_size(1024).unwrap();
        {
            let buffer = buffer.make_mut();
            let params = ExampleCMetaParams::new(COUNT, NUM);
            let _meta = ExampleCMeta::add(buffer, params);
        }
        if let Some(meta) = buffer.meta::<ExampleCMeta>() {
            assert_eq!(meta.count(), COUNT);
            assert_eq!(meta.num(), NUM);
        }
        {
            let buffer = buffer.make_mut();
            ExampleCMeta::remove(buffer).unwrap();
        }
        assert!(buffer.meta::<ExampleCMeta>().is_none());
    }
}
