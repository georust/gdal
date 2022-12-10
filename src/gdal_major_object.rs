use gdal_sys::GDALMajorObjectH;

/// Common trait for GDAL data types backed by [`GDALMajorObjectH`].
pub trait MajorObject {
    unsafe fn gdal_object_ptr(&self) -> GDALMajorObjectH;
}
