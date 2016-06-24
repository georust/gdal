use libc::{c_void};

pub trait MajorObject {
    unsafe fn gdal_object_ptr(&self) -> *const c_void;
}
