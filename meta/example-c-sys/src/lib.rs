//! ExampleCMeta binding for Rust
//!
//! ExampleCMetaをRustで扱うためのバインディング実装

use gst::{ffi::GstBuffer, prelude::*};
mod imp;

#[link(name = "example_c_meta")]
extern "C" {
    pub fn example_c_meta_get_info() -> *const gst::ffi::GstMetaInfo;
    pub fn example_c_meta_api_get_type() -> gst::glib::Type;
    #[allow(improper_ctypes)]
    pub fn buffer_add_example_c_meta(
        buffer: *mut GstBuffer,
        count: i64,
        num: f32,
    ) -> *mut imp::ExampleCMeta;
    #[allow(improper_ctypes)]
    pub fn buffer_add_param_example_c_meta(
        buffer: *mut GstBuffer,
        param: *mut ExampleCMetaParams,
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
            // 直接gst_buffer_add_metaを呼ぶ
            // let mut params = std::mem::ManuallyDrop::new(param);
            // let meta = gst::ffi::gst_buffer_add_meta(
            //     buffer.as_mut_ptr(),
            //     example_c_meta_get_info(),
            //     &mut *params as *mut ExampleCMetaParams as gst::glib::ffi::gpointer ,
            // ) as *mut imp::ExampleCMeta;

            // OK
            // 定義した関数に参照渡しで更新を指示する
            // Heapにあるデータの場合は明示的にManuallyDropが必要
            // println!(
            //     "rust size of {}",
            //     std::mem::size_of::<imp::ExampleCMetaParams>()
            // );
            let mut params = std::mem::ManuallyDrop::new(param);
            let meta = buffer_add_param_example_c_meta(
                buffer.as_mut_ptr(),
                &mut *params as *mut ExampleCMetaParams,
            );

            Self::from_mut_ptr(buffer, meta)
        }
    }

    pub fn remove(buffer: &mut gst::BufferRef) {
        if let Some(meta) = buffer.meta_mut::<Self>() {
            meta.remove().unwrap();
        }
    }

    #[doc(alias = "get_label")]
    pub fn label(&self) -> &gst::glib::GStr {
        self.0.label.as_gstr()
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
        const LABEL: &str = "hello";
        const COUNT: i64 = 12345;
        const NUM: f32 = 1.2345;
        gst::init().unwrap();
        let mut buffer = gst::Buffer::with_size(1024).unwrap();
        {
            let buffer = buffer.make_mut();
            let params = ExampleCMetaParams::new(LABEL.to_owned(), COUNT, NUM);
            let _meta = ExampleCMeta::add(buffer, params);
        }
        if let Some(meta) = buffer.meta::<ExampleCMeta>() {
            assert_eq!(meta.label().as_str(), LABEL);
            assert_eq!(meta.count(), COUNT);
            assert_eq!(meta.num(), NUM);
        }
        // TODO remove after resolve double free
        if let Some(meta) = buffer.meta::<ExampleCMeta>() {
            assert_eq!(meta.label().as_str(), LABEL);
            assert_eq!(meta.count(), COUNT);
            assert_eq!(meta.num(), NUM);
        }
        {
            let buffer = buffer.make_mut();
            ExampleCMeta::remove(buffer);
        }
        assert!(buffer.meta::<ExampleCMeta>().is_none());
    }
}
