use gdal_sys::GDALMajorObjectH;

pub trait MajorObject {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH;
}
