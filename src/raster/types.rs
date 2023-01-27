use crate::errors::{GdalError, Result};
use crate::utils::_string;
use gdal_sys::{
    GDALAdjustValueToDataType, GDALDataType, GDALDataTypeIsConversionLossy, GDALDataTypeIsFloating,
    GDALDataTypeIsInteger, GDALDataTypeIsSigned, GDALDataTypeUnion, GDALFindDataTypeForValue,
    GDALGetDataTypeByName, GDALGetDataTypeName, GDALGetDataTypeSizeBits, GDALGetDataTypeSizeBytes,
};
use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};

/// Provides ergonomic access to functions describing [`GDALDataType`] ordinals.
///
/// A [`GDALDataType`] indicates the primitive storage value of a cell/pixel in a [`RasterBand`][crate::raster::RasterBand].
///
/// # Example
/// ```rust, no_run
/// use gdal::raster::GdalType;
/// let td = <u32>::datatype();
/// println!("{} is {} and uses {} bits.",
///     td.name(),
///     if td.is_signed() { "signed" } else { "unsigned" },
///     td.bits()
/// );
/// ```
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum GdalDataType {
    /// Unknown or unspecified type
    Unknown = GDALDataType::GDT_Unknown,
    /// Eight bit unsigned integer
    UInt8 = GDALDataType::GDT_Byte,
    /// Sixteen bit unsigned integer
    UInt16 = GDALDataType::GDT_UInt16,
    /// Sixteen bit signed integer
    Int16 = GDALDataType::GDT_Int16,
    /// Thirty two bit unsigned integer
    UInt32 = GDALDataType::GDT_UInt32,
    /// Thirty two bit signed integer
    Int32 = GDALDataType::GDT_Int32,
    #[cfg(all(major_ge_3, minor_ge_5))]
    /// 64 bit unsigned integer (GDAL >= 3.5)
    UInt64 = GDALDataType::GDT_UInt64,
    #[cfg(all(major_ge_3, minor_ge_5))]
    /// 64 bit signed integer  (GDAL >= 3.5)
    Int64 = GDALDataType::GDT_Int64,
    /// Thirty two bit floating point
    Float32 = GDALDataType::GDT_Float32,
    /// Sixty four bit floating point
    Float64 = GDALDataType::GDT_Float64,
}

impl GdalDataType {
    /// Find `GdalTypeDescriptor` by name, as would be returned by [`name`][Self::name].
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::{GdalType, GdalDataType};
    /// assert_eq!(GdalDataType::from_name("UInt16").unwrap(), <u16>::datatype())
    /// ```
    pub fn from_name(name: &str) -> Result<Self> {
        let c_name = CString::new(name)?;
        let gdal_type = unsafe { GDALGetDataTypeByName(c_name.as_ptr()) };
        match gdal_type {
            GDALDataType::GDT_Unknown => Err(GdalError::BadArgument(format!(
                "unable to find datatype with name '{name}'"
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
    /// use gdal::raster::{GdalType, GdalDataType};
    /// assert_eq!(GdalDataType::for_value(0), <u8>::datatype());
    /// assert_eq!(GdalDataType::for_value(256), <u16>::datatype());
    /// assert_eq!(GdalDataType::for_value(-1), <i16>::datatype());
    /// assert_eq!(GdalDataType::for_value(<u16>::MAX as f64 * -2.0), <i32>::datatype());
    /// ```
    pub fn for_value<N: GdalType + Into<f64>>(value: N) -> Self {
        let gdal_type = unsafe { GDALFindDataTypeForValue(value.into(), 0) };
        Self::try_from(gdal_type).expect("GDALFindDataTypeForValue")
    }

    /// Get the name of the [`GDALDataType`].
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::GdalType;
    /// assert_eq!(<u16>::datatype().name(), "UInt16");
    /// ```
    pub fn name(&self) -> String {
        let c_str = unsafe { GDALGetDataTypeName(self.gdal_ordinal()) };
        if c_str.is_null() {
            // This case shouldn't happen, because `self` only exists for valid
            // GDALDataType ordinals.
            panic!(
                "GDALGetDataTypeName unexpectedly returned an empty name for {:?}",
                &self
            );
        }
        _string(c_str)
    }

    /// Get the [`GDALDataType`] size in **bits**.
    pub fn bits(&self) -> u8 {
        unsafe { GDALGetDataTypeSizeBits(self.gdal_ordinal()) }
            .try_into()
            .expect("GDALGetDataTypeSizeBits")
    }

    /// Get the [`GDALDataType`] size in **bytes**.
    pub fn bytes(&self) -> u8 {
        unsafe { GDALGetDataTypeSizeBytes(self.gdal_ordinal()) }
            .try_into()
            .expect("GDALGetDataTypeSizeBytes")
    }

    /// Returns `true` if [`GDALDataType`] is integral (non-floating point)
    pub fn is_integer(&self) -> bool {
        (unsafe { GDALDataTypeIsInteger(self.gdal_ordinal()) }) > 0
    }

    /// Returns `true` if [`GDALDataType`] is floating point (non-integral)
    pub fn is_floating(&self) -> bool {
        (unsafe { GDALDataTypeIsFloating(self.gdal_ordinal()) }) > 0
    }

    /// Returns `true` if [`GDALDataType`] supports negative values.
    pub fn is_signed(&self) -> bool {
        (unsafe { GDALDataTypeIsSigned(self.gdal_ordinal()) }) > 0
    }

    /// Return the descriptor for smallest [`GDALDataType`] that fully contains both data types
    /// indicated by `self` and `other`.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::GdalType;
    /// println!("To safely store all possible '{}' and '{}' values, you should use  '{}'",
    ///     <f32>::datatype(),
    ///     <i32>::datatype(),
    ///     <f32>::datatype().union(<i32>::datatype())
    /// );
    /// ```
    pub fn union(&self, other: Self) -> Self {
        let gdal_type = unsafe { GDALDataTypeUnion(self.gdal_ordinal(), other.gdal_ordinal()) };
        Self::try_from(gdal_type).expect("GDALDataTypeUnion")
    }

    /// Change a given value to fit within the constraints of this [`GDALDataType`].
    ///
    /// Returns an enum indicating if the wrapped value is unchanged, clamped
    /// (to min or max datatype value) or rounded (for integral data types).
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::{GdalType, AdjustedValue::*};
    /// assert_eq!(<u8>::datatype().adjust_value(255), Unchanged(255.));
    /// assert_eq!(<u8>::datatype().adjust_value(1.2334), Rounded(1.));
    /// assert_eq!(<u8>::datatype().adjust_value(1000.2334), Clamped(255.));
    /// ```
    pub fn adjust_value<N: GdalType + Into<f64>>(&self, value: N) -> AdjustedValue {
        let mut is_clamped: libc::c_int = 0;
        let mut is_rounded: libc::c_int = 0;

        let result = unsafe {
            GDALAdjustValueToDataType(
                self.gdal_ordinal(),
                value.into(),
                &mut is_clamped,
                &mut is_rounded,
            )
        };

        match (is_clamped > 0, is_rounded > 0) {
            (false, false) => AdjustedValue::Unchanged(result),
            (true, false) => AdjustedValue::Clamped(result),
            (false, true) => AdjustedValue::Rounded(result),
            (true, true) => panic!("Unexpected adjustment result: clamped and rounded."),
        }
    }

    /// Determine if converting a value from [`GDALDataType`] described by `self` to one
    /// described by `other` is potentially lossy.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::GdalType;
    /// assert!(<i16>::datatype().is_conversion_lossy(<u8>::datatype()))
    /// ```
    pub fn is_conversion_lossy(&self, other: Self) -> bool {
        let r = unsafe { GDALDataTypeIsConversionLossy(self.gdal_ordinal(), other.gdal_ordinal()) };
        r != 0
    }

    /// Subset of the GDAL data types supported by Rust bindings.
    pub fn iter() -> impl Iterator<Item = GdalDataType> {
        use GdalDataType::*;
        [
            UInt8,
            UInt16,
            Int16,
            UInt32,
            Int32,
            #[cfg(all(major_ge_3, minor_ge_5))]
            UInt64,
            #[cfg(all(major_ge_3, minor_ge_5))]
            Int64,
            Float32,
            Float64,
        ]
        .iter()
        .copied()
    }

    #[inline]
    pub(crate) fn gdal_ordinal(&self) -> GDALDataType::Type {
        *self as GDALDataType::Type
    }
}

impl Debug for GdalDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GdalTypeDescriptor")
            .field("name", &self.name())
            .field("bits", &self.bits())
            .field("signed", &self.is_signed())
            .field("floating", &self.is_floating())
            .field("gdal_ordinal", &self.gdal_ordinal())
            .finish()
    }
}

impl Display for GdalDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

/// Converts from a possible [`GDALDataType`] ordinal value to a [`GdalDataType`] (note case).
///
/// # Example
///
/// ```rust, no_run
/// use gdal::raster::GdalDataType;
/// let gdt: GdalDataType = 3.try_into().unwrap();
/// assert_eq!(gdt.name(), "Int16");
/// ```
impl TryFrom<u32> for GdalDataType {
    type Error = GdalError;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        use GDALDataType::*;
        #[allow(non_upper_case_globals)]
        match value {
            GDT_Unknown => Ok(GdalDataType::Unknown),
            GDT_Byte => Ok(GdalDataType::UInt8),
            GDT_UInt16 => Ok(GdalDataType::UInt16),
            GDT_Int16 => Ok(GdalDataType::Int16),
            GDT_UInt32 => Ok(GdalDataType::UInt32),
            GDT_Int32 => Ok(GdalDataType::Int32),
            #[cfg(all(major_ge_3, minor_ge_5))]
            GDT_UInt64 => Ok(GdalDataType::UInt64),
            #[cfg(all(major_ge_3, minor_ge_5))]
            GDT_Int64 => Ok(GdalDataType::Int64),
            GDT_Float32 => Ok(GdalDataType::Float32),
            GDT_Float64 => Ok(GdalDataType::Float64),
            GDT_CInt16 | GDT_CInt32 | GDT_CFloat32 | GDT_CFloat64 => Err(GdalError::BadArgument(
                "Complex data types are not available".into(),
            )),
            o => Err(GdalError::BadArgument(format!(
                "unknown GDALDataType ordinal' {o}'"
            ))),
        }
    }
}

/// Return type for [`GdalDataType::adjust_value`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AdjustedValue {
    /// Value was not changed
    Unchanged(f64),
    /// Value was clamped to fit within the min/max bounds of data type
    Clamped(f64),
    /// The value was rounded to fit in an integral type
    Rounded(f64),
}

impl From<AdjustedValue> for f64 {
    fn from(av: AdjustedValue) -> Self {
        match av {
            AdjustedValue::Unchanged(v) => v,
            AdjustedValue::Clamped(v) => v,
            AdjustedValue::Rounded(v) => v,
        }
    }
}

/// Type-level constraint for bounding primitive numeric values for generic
/// functions requiring a [`GDALDataType`].
///
/// See [`GdalDataType`] for access to metadata describing the data type.
pub trait GdalType {
    /// Get the [`GDALDataType`] ordinal value used in `gdal_sys` to represent a GDAL cell/pixel
    /// data type.
    ///
    /// See also: [GDAL API](https://gdal.org/api/raster_c_api.html#_CPPv412GDALDataType)
    fn gdal_ordinal() -> GDALDataType::Type;

    /// Get the metadata type over a `GdalType`.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::raster::GdalType;
    /// let gdt = <u32>::datatype();
    /// println!("{gdt:#?}");
    /// ```
    fn datatype() -> GdalDataType {
        // We can call `unwrap` because existence is guaranteed in this case.
        Self::gdal_ordinal()
            .try_into()
            .unwrap_or(GdalDataType::Unknown)
    }
}

/// Provides evidence `u8` is a valid [`GDALDataType`].
impl GdalType for u8 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_Byte
    }
}

/// Provides evidence `u16` is a valid [`GDALDataType`].
impl GdalType for u16 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_UInt16
    }
}

/// Provides evidence `u32` is a valid [`GDALDataType`].
impl GdalType for u32 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_UInt32
    }
}

#[cfg(all(major_ge_3, minor_ge_5))]
/// Provides evidence `u64` is a valid [`GDALDataType`].
impl GdalType for u64 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_UInt64
    }
}

/// Provides evidence `i16` is a valid [`GDALDataType`].
impl GdalType for i16 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_Int16
    }
}

/// Provides evidence `i32` is a valid [`GDALDataType`].
impl GdalType for i32 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_Int32
    }
}

#[cfg(all(major_ge_3, minor_ge_5))]
/// Provides evidence `i64` is a valid [`GDALDataType`].
impl GdalType for i64 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_Int64
    }
}

/// Provides evidence `f32` is a valid [`GDALDataType`].
impl GdalType for f32 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_Float32
    }
}

/// Provides evidence `f64` is a valid [`GDALDataType`].
impl GdalType for f64 {
    fn gdal_ordinal() -> GDALDataType::Type {
        GDALDataType::GDT_Float64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::types::AdjustedValue::{Clamped, Rounded, Unchanged};
    use gdal_sys::GDALDataType::*;

    #[test]
    #[allow(non_upper_case_globals)]
    fn test_gdal_data_type() {
        for t in GdalDataType::iter() {
            // Test converting from GDALDataType:Type
            let t2: GdalDataType = t.gdal_ordinal().try_into().unwrap();
            assert_eq!(&t, &t2, "{t}");
            assert!(t.bits() > 0, "{}", t);
            assert_eq!(t.bits(), t.bytes() * 8, "{t}");
            let name = t.name();
            match t.gdal_ordinal() {
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

                o => panic!("unknown type ordinal '{o}'"),
            }
            match t.gdal_ordinal() {
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
                o => panic!("unknown type ordinal '{o}'"),
            }
        }

        for t in [GDT_CInt16, GDT_CInt32, GDT_CFloat32, GDT_CFloat64] {
            assert!(TryInto::<GdalDataType>::try_into(t).is_err());
        }
    }

    #[test]
    fn test_data_type_from_name() {
        assert!(GdalDataType::from_name("foobar").is_err());

        for t in GdalDataType::iter() {
            let name = t.name();
            let t2 = GdalDataType::from_name(&name);
            assert!(t2.is_ok());
        }
    }

    #[test]
    fn test_data_type_union() {
        let f32d = <f32>::datatype();
        let f64d = <f64>::datatype();

        let u8d = <u8>::datatype();
        let u16d = <u16>::datatype();
        let i16d = <i16>::datatype();
        let i32d = <i32>::datatype();

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
            let u32d = <u32>::datatype();
            let i64d = <i64>::datatype();
            assert_eq!(i16d.union(u32d), i64d);
        }
    }

    #[test]
    fn test_for_value() {
        assert_eq!(GdalDataType::for_value(0), <u8>::datatype());
        assert_eq!(GdalDataType::for_value(256), <u16>::datatype());
        assert_eq!(GdalDataType::for_value(-1), <i16>::datatype());
        assert_eq!(
            GdalDataType::for_value(<u16>::MAX as f64 * -2.0),
            <i32>::datatype()
        );
    }

    #[test]
    fn test_adjust_value() {
        assert_eq!(<u8>::datatype().adjust_value(255), Unchanged(255.));
        assert_eq!(<u8>::datatype().adjust_value(1.2334), Rounded(1.));
        assert_eq!(<u8>::datatype().adjust_value(1000.2334), Clamped(255.));
        assert_eq!(<u8>::datatype().adjust_value(-1), Clamped(0.));
        assert_eq!(<i16>::datatype().adjust_value(-32768), Unchanged(-32768.0));
        assert_eq!(<i16>::datatype().adjust_value(-32767.4), Rounded(-32767.0));
        assert_eq!(
            <f32>::datatype().adjust_value(1e300),
            Clamped(f32::MAX as f64)
        );
        let v: f64 = <i16>::datatype().adjust_value(-32767.4).into();
        assert_eq!(v, -32767.0);
    }
}
