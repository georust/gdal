use crate::errors::{GdalError, Result};
use crate::utils::{_last_null_pointer_err, _string};
pub use gdal_sys::GDALDataType;
use gdal_sys::{
    GDALDataTypeIsFloating, GDALDataTypeIsInteger, GDALDataTypeIsSigned, GDALGetDataTypeName,
    GDALGetDataTypeSizeBits, GDALGetDataTypeSizeBytes,
};
use std::fmt::{Display, Formatter};

/// Type-level constraint for limiting which primitive numeric values can be passed
/// to functions needing target data type.
pub trait GdalType {
    fn gdal_type() -> GDALDataType::Type;
    fn descriptor() -> GdalTypeDescriptor {
        // We can call `unwrap` because existence is guaranteed in this case.
        Self::gdal_type().try_into().unwrap()
    }
}

impl GdalType for u8 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Byte
    }
}

impl GdalType for u16 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_UInt16
    }
}

impl GdalType for u32 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_UInt32
    }
}

impl GdalType for i16 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Int16
    }
}

impl GdalType for i32 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Int32
    }
}

impl GdalType for f32 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Float32
    }
}

impl GdalType for f64 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Float64
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GdalTypeDescriptor(GDALDataType::Type);

impl GdalTypeDescriptor {
    pub fn gdal_type(&self) -> GDALDataType::Type {
        self.0
    }
    pub fn name(&self) -> Result<String> {
        let c_str = unsafe { GDALGetDataTypeName(self.gdal_type()) };
        if c_str.is_null() {
            return Err(_last_null_pointer_err("GDALGetDescription"));
        }
        Ok(_string(c_str))
    }

    /// Get the gdal type size in **bits**.
    pub fn bits(&self) -> u8 {
        unsafe { GDALGetDataTypeSizeBits(self.gdal_type()) }
            .try_into()
            .unwrap()
    }

    /// Get the gdal type size in **bytes**.
    pub fn bytes(&self) -> u8 {
        unsafe { GDALGetDataTypeSizeBytes(self.gdal_type()) }
            .try_into()
            .unwrap()
    }

    /// Returns `true` if data type is integral (non-floating point)
    pub fn is_integer(&self) -> bool {
        (unsafe { GDALDataTypeIsInteger(self.gdal_type()) }) > 0
    }

    /// Returns `true` if data type is floating point (non-integral)
    pub fn is_floating(&self) -> bool {
        (unsafe { GDALDataTypeIsFloating(self.gdal_type()) }) > 0
    }

    /// Returns `true` if data type supports negative values.
    pub fn is_signed(&self) -> bool {
        (unsafe { GDALDataTypeIsSigned(self.gdal_type()) }) > 0
    }

    /// Subset of the GDAL data types supported by Rust bindings.
    pub fn available_types() -> &'static [GdalTypeDescriptor] {
        use GDALDataType::*;
        &[
            GdalTypeDescriptor(GDT_Byte),
            GdalTypeDescriptor(GDT_UInt16),
            GdalTypeDescriptor(GDT_Int16),
            GdalTypeDescriptor(GDT_UInt32),
            GdalTypeDescriptor(GDT_Int32),
            GdalTypeDescriptor(GDT_UInt64),
            GdalTypeDescriptor(GDT_Int64),
            GdalTypeDescriptor(GDT_Float32),
            GdalTypeDescriptor(GDT_Float64),
        ]
    }
}

impl Display for GdalTypeDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name().unwrap())
    }
}

impl TryFrom<GDALDataType::Type> for GdalTypeDescriptor {
    type Error = GdalError;

    fn try_from(value: GDALDataType::Type) -> std::result::Result<Self, Self::Error> {
        let wrapped = GdalTypeDescriptor(value);
        if !GdalTypeDescriptor::available_types().contains(&wrapped) {
            Err(GdalError::BadArgument(format!(
                "unknown GDALDataType {value}"
            )))
        } else {
            Ok(wrapped)
        }
    }
}
