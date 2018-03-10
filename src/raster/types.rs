use gdal_sys::GDALDataType;

pub trait GdalType {
    fn gdal_type() -> GDALDataType::Type;
}

impl GdalType for u8    { fn gdal_type() -> GDALDataType::Type { GDALDataType::GDT_Byte } }
impl GdalType for u16   { fn gdal_type() -> GDALDataType::Type { GDALDataType::GDT_UInt16 } }
impl GdalType for u32   { fn gdal_type() -> GDALDataType::Type { GDALDataType::GDT_UInt32 } }
impl GdalType for i16   { fn gdal_type() -> GDALDataType::Type { GDALDataType::GDT_Int16 } }
impl GdalType for i32   { fn gdal_type() -> GDALDataType::Type { GDALDataType::GDT_Int32 } }
impl GdalType for f32   { fn gdal_type() -> GDALDataType::Type { GDALDataType::GDT_Float32 } }
impl GdalType for f64   { fn gdal_type() -> GDALDataType::Type { GDALDataType::GDT_Float64 } }
