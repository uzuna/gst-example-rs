use gst::glib::translate::IntoGlib;
use once_cell::sync::Lazy;
use std::ptr;

use crate::METAAPINAME;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    A,
    B,
    C,
}

impl Default for Mode {
    fn default() -> Self {
        Self::A
    }
}

#[derive(Debug, Default)]
pub struct ExampleRsMetaParams {
    pub label: String,
    pub index: i32,
    pub mode: Mode,
}

impl ExampleRsMetaParams {
    pub fn new(label: String,index: i32, mode: Mode) -> Self{
        Self { label, index, mode }
    }    
}

// This is the C type that is actually stored as meta inside the buffers.
#[repr(C)]
pub struct ExampleRsMeta {
    parent: gst::ffi::GstMeta,
    pub label: String,
    pub index: i32,
    pub mode: Mode,
}

impl ExampleRsMeta {
    pub fn clone_params(&self) -> ExampleRsMetaParams {
        ExampleRsMetaParams {
            label: self.label.clone(),
            index: self.index,
            mode: self.mode,
        }
    }
}

// meta APIにメタデータのtypeを返す関数を登録
#[no_mangle]
pub unsafe extern "C" fn example_rs_meta_api_get_type() -> gst::glib::Type {
    static TYPE: Lazy<gst::glib::Type> = Lazy::new(|| unsafe {
        let t = gst::glib::translate::from_glib(gst::ffi::gst_meta_api_type_register(
            METAAPINAME.as_ptr() as *const _,
            [ptr::null::<std::os::raw::c_char>()].as_ptr() as *mut *const _,
        ));

        assert_ne!(t, gst::glib::Type::INVALID);
        t
    });

    *TYPE
}
// メタデータの領域を確保して初期化を行う関数
#[no_mangle]
pub unsafe extern "C" fn example_rs_meta_init(
    meta: *mut gst::ffi::GstMeta,
    params: gst::glib::ffi::gpointer,
    _buffer: *mut gst::ffi::GstBuffer,
) -> gst::glib::ffi::gboolean {
    assert!(!params.is_null());

    let meta = &mut *(meta as *mut ExampleRsMeta);
    let params = ptr::read(params as *const ExampleRsMetaParams);

    // Need to initialize all our fields correctly here.
    ptr::write(&mut meta.label, params.label);
    ptr::write(&mut meta.index, params.index);
    ptr::write(&mut meta.mode, params.mode);

    true.into_glib()
}

// メタデータ開放時に呼ばれる関数
#[no_mangle]
pub unsafe extern "C" fn example_rs_meta_free(
    meta: *mut gst::ffi::GstMeta,
    _buffer: *mut gst::ffi::GstBuffer,
) {
    let meta = &mut *(meta as *mut ExampleRsMeta);

    // ヒープにある情報は明示的に開放する
    ptr::drop_in_place(&mut meta.label);
}

// エレメントで古いバッファから新しいバッファにコピーする時に呼ばれる関数
#[no_mangle]
pub unsafe extern "C" fn example_rs_meta_transform(
    dest: *mut gst::ffi::GstBuffer,
    meta: *mut gst::ffi::GstMeta,
    _buffer: *mut gst::ffi::GstBuffer,
    _type_: gst::glib::ffi::GQuark,
    _data: gst::glib::ffi::gpointer,
) -> gst::glib::ffi::gboolean {
    let meta = &*(meta as *mut ExampleRsMeta);

    // この例ではシンプルにデータをコピーする
    // メタデータの種類、バッファとのつながりを考慮してコピーや削除をする
    super::ExampleRsMeta::add(gst::BufferRef::from_mut_ptr(dest), meta.clone_params());

    true.into_glib()
}

// Register the meta itself with its functions.
#[no_mangle]
pub fn example_rs_meta_get_info() -> *const gst::ffi::GstMetaInfo {
    struct MetaInfo(ptr::NonNull<gst::ffi::GstMetaInfo>);
    unsafe impl Send for MetaInfo {}
    unsafe impl Sync for MetaInfo {}

    static META_INFO: Lazy<MetaInfo> = Lazy::new(|| unsafe {
        MetaInfo(
            ptr::NonNull::new(gst::ffi::gst_meta_register(
                example_rs_meta_api_get_type().into_glib(),
                crate::METANAME.as_ptr() as *const _,
                std::mem::size_of::<ExampleRsMeta>(),
                Some(example_rs_meta_init),
                Some(example_rs_meta_free),
                Some(example_rs_meta_transform),
            ) as *mut gst::ffi::GstMetaInfo)
            .expect("Failed to register meta API"),
        )
    });

    META_INFO.0.as_ptr()
}
