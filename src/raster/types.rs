use crate::errors::{GdalError, Result};
use crate::utils::{_last_null_pointer_err, _string};
pub use gdal_sys::GDALDataType;
use gdal_sys::{
    GDALDataTypeIsFloating, GDALDataTypeIsInteger, GDALDataTypeIsSigned, GDALDataTypeUnion,
    GDALGetDataTypeByName, GDALGetDataTypeName, GDALGetDataTypeSizeBits, GDALGetDataTypeSizeBytes,
};
use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct GdalTypeDescriptor(GDALDataType::Type);

impl GdalTypeDescriptor {
    /// Find `GDALDataType` by name, as would be returned by [`GdalTypeDescriptor.name()`].
    #[allow(non_snake_case)]
    pub fn from_name(name: &str) -> Result<Self> {
        let c_name = CString::new(name.to_owned())?;
        let gdal_type = unsafe { GDALGetDataTypeByName(c_name.as_ptr()) };
        match gdal_type {
            GDALDataType::GDT_Unknown => Err(GdalError::BadArgument(format!(
                "unable to find datatype with name '{}'",
                name
            ))),
            _ => gdal_type.try_into(),
        }
    }

    /// Get the `GDALDataType` ordinal value
    pub fn gdal_type(&self) -> GDALDataType::Type {
        self.0
    }

    /// Get the name of the `GDALDataType`.
    pub fn name(&self) -> Result<String> {
        let c_str = unsafe { GDALGetDataTypeName(self.gdal_type()) };
        if c_str.is_null() {
            return Err(_last_null_pointer_err("GDALGetDataTypeName"));
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

    /// Return the smallest data type that can fully express both `self` and
    /// `other` data types.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gdal::raster::GdalType;
    /// assert_eq!(<f32>::descriptor().union(<i32>::descriptor()), <f64>::descriptor());
    /// ```
    pub fn union(&self, other: Self) -> Self {
        let gdal_type = unsafe { GDALDataTypeUnion(self.gdal_type(), other.gdal_type()) };
        Self(gdal_type)
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
            #[cfg(all(major_ge_3, minor_ge_5))]
            GdalTypeDescriptor(GDT_UInt64),
            #[cfg(all(major_ge_3, minor_ge_5))]
            GdalTypeDescriptor(GDT_Int64),
            GdalTypeDescriptor(GDT_Float32),
            GdalTypeDescriptor(GDT_Float64),
        ]
    }
}

impl Debug for GdalTypeDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GdalTypeDescriptor")
            .field("name", &self.name().unwrap_or_else(|e| format!("{e:?}")))
            .field("bits", &self.bits())
            .field("signed", &self.is_signed())
            .field("gdal_ordinal", &self.gdal_type())
            .finish()
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

/// Type-level constraint for bounding primitive numeric values passed
/// to functions requiring a data type. See [`GdalTypeDescriptor`] for access to
/// metadata describing the data type.
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

#[cfg(all(major_ge_3, minor_ge_5))]
impl GdalType for u64 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_UInt64
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

#[cfg(all(major_ge_3, minor_ge_5))]
impl GdalType for i64 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Int64
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
