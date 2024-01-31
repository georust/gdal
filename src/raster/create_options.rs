use crate::cpl::CslStringList;

/// Key/value pairs of options for passing driver-specific creation flags to
/// [`Driver::create_with_band_type_with_options`](crate::Driver::create_with_band_type_with_options`).
///
/// See `papszOptions` in [GDAL's `Create(...)` API documentation](https://gdal.org/api/gdaldriver_cpp.html#_CPPv4N10GDALDriver6CreateEPKciii12GDALDataType12CSLConstList).
pub type RasterCreationOptions = CslStringList;
