use crate::errors::{GdalError, Result};
use crate::utils::{_last_null_pointer_err, _string};
pub use gdal_sys::GDALDataType;
use gdal_sys::{
    GDALAdjustValueToDataType, GDALDataTypeIsFloating, GDALDataTypeIsInteger, GDALDataTypeIsSigned,
    GDALDataTypeUnion, GDALFindDataTypeForValue, GDALGetDataTypeByName, GDALGetDataTypeName,
    GDALGetDataTypeSizeBits, GDALGetDataTypeSizeBytes,
};
use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};

/// Provides ergonomic access to functions describing [`GDALDataType`] ordinals.
///
/// A [`GDALDataType`] indicates the primitive storage value of a cell/pixel in a [`RasterBand`][crate::raster::RasterBand].
///
/// # Example
/// ```rust, no_run
/// use gdal::raster::{GdalType, GdalTypeDescriptor};
/// let td = <u32>::descriptor();
/// println!("{} is {} and uses {} bits.",
///     td.name(),
///     if td.is_signed() { "signed" } else { "unsigned" },
///     td.bits()
/// );
/// ```
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct GdalTypeDescriptor(GDALDataType::Type);

impl GdalTypeDescriptor {
    /// Find `GdalTypeDescriptor` by name, as would be returned by [`GdalTypeDescriptor.name()`].
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::{GdalType, GdalTypeDescriptor};
    /// assert_eq!(GdalTypeDescriptor::from_name("UInt16").unwrap().gdal_type(), <u16>::gdal_type())
    /// ```
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

    /// Finds the smallest data type able to support the provided value.
    ///
    /// See [`GDALFindDataTypeForValue`](https://gdal.org/api/raster_c_api.html#_CPPv424GDALFindDataTypeForValuedi)
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::{GdalType, GdalTypeDescriptor};
    /// assert_eq!(GdalTypeDescriptor::for_value(0), <u8>::descriptor());
    /// assert_eq!(GdalTypeDescriptor::for_value(256), <u16>::descriptor());
    /// assert_eq!(GdalTypeDescriptor::for_value(-1), <i16>::descriptor());
    /// assert_eq!(GdalTypeDescriptor::for_value(<u16>::MAX as f64 * -2.0), <i32>::descriptor());
    /// ```
    pub fn for_value<N: GdalType + Into<f64>>(value: N) -> Self {
        let gdal_type = unsafe { GDALFindDataTypeForValue(value.into(), 0) };
        GdalTypeDescriptor(gdal_type)
    }

    /// Get the `GDALDataType` ordinal value
    pub fn gdal_type(&self) -> GDALDataType::Type {
        self.0
    }

    /// Get the name of the `GDALDataType`.
    pub fn name(&self) -> String {
        let c_str = unsafe { GDALGetDataTypeName(self.gdal_type()) };
        if c_str.is_null() {
            // This case shouldn't happen, because `self` only exists for valid
            // GDALDataType ordinals.
            panic!("{}", _last_null_pointer_err("GDALGetDataTypeName"));
        }
        _string(c_str)
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
    /// ```rust, no_run
    /// use gdal::raster::GdalType;
    /// println!("To safely store all possible '{}' and '{}' values, you should use  '{}'",
    ///     <f32>::descriptor(),
    ///     <i32>::descriptor(),
    ///     <f32>::descriptor().union(<i32>::descriptor())
    /// );
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
            .field("name", &self.name())
            .field("bits", &self.bits())
            .field("signed", &self.is_signed())
            .field("floating", &self.is_floating())
            .field("gdal_ordinal", &self.gdal_type())
            .finish()
    }
}

impl Display for GdalTypeDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

/// Converts from a possible [`GDALDataType`] ordinal value to a [`GdalTypeDescriptor`].
///
/// # Example
///
/// ```rust, no_run
/// use gdal::raster::{GdalType, GdalTypeDescriptor};
/// let gdt: GdalTypeDescriptor = 3.try_into().unwrap();
/// println!("{gdt:#?}")
/// ```
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AdjustedValue {
    Unchanged(f64),
    Clamped(f64),
    Rounded(f64),
    ClampedRounded(f64),
}

impl From<AdjustedValue> for f64 {
    fn from(av: AdjustedValue) -> Self {
        match av {
            AdjustedValue::Unchanged(v) => v,
            AdjustedValue::Clamped(v) => v,
            AdjustedValue::Rounded(v) => v,
            AdjustedValue::ClampedRounded(v) => v,
        }
    }
}

/// Type-level constraint for bounding primitive numeric values for generic
/// functions requiring a data type.
///
/// See [`GdalTypeDescriptor`] for access to metadata describing the data type.
pub trait GdalType {
    /// Get the [`GDALDataType`] ordinal value used in `gdal_sys` to represent a GDAL cell/pixel
    /// data type.
    ///
    /// See also: [GDAL API](https://gdal.org/api/raster_c_api.html#_CPPv412GDALDataType)
    fn gdal_type() -> GDALDataType::Type;

    /// Get the metadata type over a `GdalType`.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::GdalType;
    /// let gdt = <u32>::descriptor();
    /// println!("{gdt:#?}");
    /// ```
    fn descriptor() -> GdalTypeDescriptor {
        // We can call `unwrap` because existence is guaranteed in this case.
        Self::gdal_type().try_into().unwrap()
    }
}

/// Provides evidence `u8` is a valid [`GDALDataType`].
impl GdalType for u8 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Byte
    }
}

/// Provides evidence `u16` is a valid [`GDALDataType`].
impl GdalType for u16 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_UInt16
    }
}

/// Provides evidence `u32` is a valid [`GDALDataType`].
impl GdalType for u32 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_UInt32
    }
}

#[cfg(all(major_ge_3, minor_ge_5))]
/// Provides evidence `u64` is a valid [`GDALDataType`].
impl GdalType for u64 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_UInt64
    }
}

/// Provides evidence `i16` is a valid [`GDALDataType`].
impl GdalType for i16 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Int16
    }
}

/// Provides evidence `i32` is a valid [`GDALDataType`].
impl GdalType for i32 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Int32
    }
}

#[cfg(all(major_ge_3, minor_ge_5))]
/// Provides evidence `i64` is a valid [`GDALDataType`].
impl GdalType for i64 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Int64
    }
}

/// Provides evidence `f32` is a valid [`GDALDataType`].
impl GdalType for f32 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Float32
    }
}

/// Provides evidence `f64` is a valid [`GDALDataType`].
impl GdalType for f64 {
    fn gdal_type() -> GDALDataType::Type {
        GDALDataType::GDT_Float64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdal_sys::GDALDataType::*;
    use crate::raster::types::AdjustedValue::{ClampedRounded, Rounded};

    #[test]
    #[allow(non_upper_case_globals)]
    fn test_gdal_data_type() {
        for t in GdalTypeDescriptor::available_types() {
            // Test converting from GDALDataType:Type
            let t2: GdalTypeDescriptor = t.gdal_type().try_into().unwrap();
            assert_eq!(t, &t2, "{}", t);
            assert!(t.bits() > 0, "{}", t);
            assert_eq!(t.bits(), t.bytes() * 8, "{}", t);
            let name = t.name();
            match t.gdal_type() {
                GDT_Byte | GDT_UInt16 | GDT_Int16 | GDT_UInt32 | GDT_Int32 => {
                    assert!(t.is_integer(), "{}", &name);
                    assert!(!t.is_floating(), "{}", &name);
                }
                #[cfg(all(major_ge_3, minor_ge_5))]
                GDT_UInt64 | GDT_Int64 => {
                    assert!(t.is_integer(), "{}", &name);
                    assert!(!t.is_floating(), "{}", &name);
                }
                GDT_Float32 | GDT_Float64 => {
                    assert!(!t.is_integer(), "{}", &name);
                    assert!(t.is_floating(), "{}", &name);
                }

                o => panic!("unknown type ordinal '{}'", o),
            }
            match t.gdal_type() {
                GDT_Byte | GDT_UInt16 | GDT_UInt32 => {
                    assert!(!t.is_signed(), "{}", &name);
                }
                #[cfg(all(major_ge_3, minor_ge_5))]
                GDT_UInt64 => {
                    assert!(!t.is_signed(), "{}", &name);
                }
                GDT_Int16 | GDT_Int32 | GDT_Float32 | GDT_Float64 => {
                    assert!(t.is_signed(), "{}", &name);
                }
                #[cfg(all(major_ge_3, minor_ge_5))]
                GDT_Int64 => {
                    assert!(t.is_signed(), "{}", &name);
                }
                o => panic!("unknown type ordinal '{}'", o),
            }
        }
    }

    #[test]
    fn test_data_type_from_name() {
        assert!(GdalTypeDescriptor::from_name("foobar").is_err());

        for t in GdalTypeDescriptor::available_types() {
            let name = t.name();
            let t2 = GdalTypeDescriptor::from_name(&name);
            assert!(t2.is_ok());
        }
    }

    #[test]
    fn test_data_type_union() {
        let f32d = <f32>::descriptor();
        let f64d = <f64>::descriptor();

        let u8d = <u8>::descriptor();
        let u16d = <u16>::descriptor();
        let i16d = <i16>::descriptor();
        let i32d = <i32>::descriptor();

        // reflexivity
        assert_eq!(i16d.union(i16d), i16d);
        // symmetry
        assert_eq!(i16d.union(f32d), f32d);
        assert_eq!(f32d.union(i16d), f32d);
        // widening
        assert_eq!(u8d.union(u16d), u16d);
        assert_eq!(f32d.union(i32d), f64d);

        #[cfg(all(major_ge_3, minor_ge_5))]
        {
            let u32d = <u32>::descriptor();
            let i64d = <i64>::descriptor();
            assert_eq!(i16d.union(u32d), i64d);
        }
    }

    #[test]
    fn test_for_value() {
        assert_eq!(GdalTypeDescriptor::for_value(0), <u8>::descriptor());
        assert_eq!(GdalTypeDescriptor::for_value(256), <u16>::descriptor());
        assert_eq!(GdalTypeDescriptor::for_value(-1), <i16>::descriptor());
        assert_eq!(
            GdalTypeDescriptor::for_value(<u16>::MAX as f64 * -2.0),
            <i32>::descriptor()
        );
    }
}
