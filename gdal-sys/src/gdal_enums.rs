use libc::{c_int};

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
#[repr(C)]
pub enum GDALDataType {
    GDT_Unknown = 0,    // Unknown or unspecified type
    GDT_Byte,       // Eight bit unsigned integer
    GDT_UInt16,     // Sixteen bit unsigned integer
    GDT_Int16,      // Sixteen bit signed integer
    GDT_UInt32,     // Thirty two bit unsigned integer
    GDT_Int32,      // Thirty two bit signed integer
    GDT_Float32,    // Thirty two bit floating point
    GDT_Float64,    // Sixty four bit floating point
    GDT_CInt16,     // Complex Int16
    GDT_CInt32,     // Complex Int32
    GDT_CFloat32,   // Complex Float32
    GDT_CFloat64,   // Complex Float64
}

impl GDALDataType {
    pub fn from_c_int(gdal_type: c_int) -> GDALDataType {
        match gdal_type {
            gdal_type if gdal_type == GDALDataType::GDT_Byte     as c_int => GDALDataType::GDT_Byte,
            gdal_type if gdal_type == GDALDataType::GDT_UInt16   as c_int => GDALDataType::GDT_UInt16,
            gdal_type if gdal_type == GDALDataType::GDT_UInt32   as c_int => GDALDataType::GDT_UInt32,
            gdal_type if gdal_type == GDALDataType::GDT_Int32    as c_int => GDALDataType::GDT_Int32,
            gdal_type if gdal_type == GDALDataType::GDT_Float32  as c_int => GDALDataType::GDT_Float32,
            gdal_type if gdal_type == GDALDataType::GDT_Float64  as c_int => GDALDataType::GDT_Float64,
            gdal_type if gdal_type == GDALDataType::GDT_CInt16   as c_int => GDALDataType::GDT_CInt16,
            gdal_type if gdal_type == GDALDataType::GDT_CInt32   as c_int => GDALDataType::GDT_CInt32,
            gdal_type if gdal_type == GDALDataType::GDT_CFloat32 as c_int => GDALDataType::GDT_CFloat32,
            gdal_type if gdal_type == GDALDataType::GDT_CFloat64 as c_int => GDALDataType::GDT_CFloat64,
            _ => GDALDataType::GDT_Unknown
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[repr(C)]
pub enum GDALRWFlag {
    GF_Read,    //Read data
    GF_Write,   //Write data
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[repr(C)]
pub enum GDALAccess {
    GA_ReadOnly,    //Read only (no update) access
    GA_Update,      //Read/write access.
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[repr(C)]
pub enum GDALResampleAlg {
    GRA_NearestNeighbour,   //Nearest neighbour (select on one input pixel)
    GRA_Bilinear,           //Bilinear (2x2 kernel)
    GRA_Cubic,              //Cubic Convolution Approximation (4x4 kernel)
    GRA_CubicSpline,        //Cubic B-Spline Approximation (4x4 kernel)
    GRA_Lanczos,            //Lanczos windowed sinc interpolation (6x6 kernel)
    GRA_Average,            //Average (computes the average of all non-NODATA contributing pixels)
    GRA_Mode,               //Mode (selects the value which appears most often of all the sampled points)
    GRA_Max,                //Max (selects maximum of all non-NODATA contributing pixels)
    GRA_Min,                //Min (selects minimum of all non-NODATA contributing pixels)
    GRA_Med,                //Med (selects median of all non-NODATA contributing pixels)
    GRA_Q1,                 //Q1 (selects first quartile of all non-NODATA contributing pixels)
    GRA_Q3,                 //Q3 (selects third quartile of all non-NODATA contributing pixels)
}
