use libc::{c_void};

pub trait MajorObject {
    unsafe fn get_gdal_object_ptr(&self) -> *const c_void;
}
