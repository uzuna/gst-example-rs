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

#[repr(C)]
pub struct ExampleRsMeta {
    parent: gst::ffi::GstMeta,
    pub label: String,
    pub index: i32,
    pub mode: Mode,
}
