use crate::{SpatialRef, SpatialRefCommon};

#[allow(non_camel_case_types)]
pub trait SpatialRef_3_0: SpatialRefCommon {

    fn set_axis_mapping_strategy(&self, strategy: gdal_sys::OSRAxisMappingStrategy::Type) {
        unsafe {
            gdal_sys::OSRSetAxisMappingStrategy(self.c_spatial_ref(), strategy);
        }
    }

    fn get_axis_mapping_strategy(&self) -> gdal_sys::OSRAxisMappingStrategy::Type {
        unsafe { gdal_sys::OSRGetAxisMappingStrategy(self.c_spatial_ref()) }
    }
}

impl SpatialRef_3_0 for SpatialRef {}
