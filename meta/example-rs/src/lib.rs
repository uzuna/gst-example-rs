use gst::prelude::*;

const METANAME: &[u8] = b"ExampleRsMeta\0";
const METAAPINAME: &[u8] = b"ExampleRsMetaAPI\0";

mod imp;

// Public Rust type for the custom meta.
#[repr(transparent)]
pub struct ExampleRsMeta(imp::ExampleRsMeta);
pub use imp::ExampleRsMetaParams;
use imp::Mode;

// Metas must be Send+Sync.
unsafe impl Send for ExampleRsMeta {}
unsafe impl Sync for ExampleRsMeta {}

unsafe impl MetaAPI for ExampleRsMeta {
    type GstType = imp::ExampleRsMeta;

    fn meta_api() -> gst::glib::Type {
        unsafe { imp::example_rs_meta_api_get_type() }
    }
}

impl ExampleRsMeta {
    // 別のバッファにデータをクローンするメソッド
    pub fn add(
        buffer: &mut gst::BufferRef,
        refmeta: imp::ExampleRsMetaParams,
    ) -> gst::MetaRefMut<Self, gst::meta::Standalone> {
        unsafe {
            let mut params = std::mem::ManuallyDrop::new(refmeta);
            let meta = gst::ffi::gst_buffer_add_meta(
                buffer.as_mut_ptr(),
                imp::example_rs_meta_get_info(),
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
    pub fn mode(&self) -> Mode {
        self.0.mode
    }
}

#[cfg(test)]
mod tests {
    use crate::{imp::Mode, ExampleRsMeta, ExampleRsMetaParams};
    #[test]
    fn test_write_read() {
        const LABEL: &str = "testlabel";
        const INDEX: i32 = 12345;
        const MODE: Mode = Mode::C;

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
