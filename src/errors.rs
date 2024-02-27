//! GDAL Error Types

use libc::c_int;
use std::num::TryFromIntError;
use thiserror::Error;

use gdal_sys::{CPLErr, OGRErr, OGRFieldType, OGRwkbGeometryType};

pub type Result<T> = std::result::Result<T, GdalError>;

#[derive(Clone, Debug, Error)]
pub enum GdalError {
    #[error("FfiNulError")]
    FfiNulError(#[from] std::ffi::NulError),
    #[error("FfiIntoStringError")]
    FfiIntoStringError(#[from] std::ffi::IntoStringError),
    #[error("StrUtf8Error")]
    StrUtf8Error(#[from] std::str::Utf8Error),
    #[cfg(feature = "ndarray")]
    #[cfg_attr(docsrs, doc(cfg(feature = "array")))]
    #[error("NdarrayShapeError")]
    NdarrayShapeError(#[from] ndarray::ShapeError),
    #[error("CPL error class: '{class:?}', error number: '{number}', error msg: '{msg}'")]
    CplError {
        class: CPLErr::Type,
        number: c_int,
        msg: String,
    },
    #[error("GDAL method '{method_name}' returned a NULL pointer. Error msg: '{msg}'")]
    NullPointer {
        method_name: &'static str,
        msg: String,
    },
    #[error("Can't cast to f64")]
    CastToF64Error,
    #[error("OGR method '{method_name}' returned error: '{err:?}'")]
    OgrError {
        err: OGRErr::Type,
        method_name: &'static str,
    },
    #[error("Unhandled type '{field_type:?}' on OGR method {method_name}")]
    UnhandledFieldType {
        field_type: OGRFieldType::Type,
        method_name: &'static str,
    },
    #[error("Invalid field name '{field_name}' used on method {method_name}")]
    InvalidFieldName {
        field_name: String,
        method_name: &'static str,
    },
    #[error("Invalid field index '{index}' used on method '{method_name}'")]
    InvalidFieldIndex {
        index: usize,
        method_name: &'static str,
    },
    #[error("Unlinked Geometry on method '{method_name}'")]
    UnlinkedGeometry { method_name: &'static str },
    #[error(
        "Invalid coordinate range while transforming points from '{from}' to '{to}': '{msg:?}'"
    )]
    InvalidCoordinateRange {
        from: String,
        to: String,
        msg: Option<String>,
    },
    #[error("Axis not found for key '{key}' in method '{method_name}'")]
    AxisNotFoundError {
        key: String,
        method_name: &'static str,
    },
    #[error("Unsupported GDAL geometry type")]
    UnsupportedGdalGeometryType(OGRwkbGeometryType::Type),
    #[error("Unable to unlink mem file: {file_name}")]
    UnlinkMemFile { file_name: String },
    #[error("BadArgument")]
    BadArgument(String),
    #[error("Date conversion error: {0}")]
    DateError(String),

    #[cfg(all(major_ge_3, minor_ge_1))]
    #[error("Unhandled type '{data_type}' on GDAL MD method {method_name}")]
    UnsupportedMdDataType {
        data_type: crate::raster::ExtendedDataTypeClass,
        method_name: &'static str,
    },
    #[error(transparent)]
    IntConversionError(#[from] TryFromIntError),
    #[error("Buffer length {0} does not match raster size {1:?}")]
    BufferSizeMismatch(usize, (usize, usize)),
    #[error("An unexpected logic error has occurred: {0}")]
    UnexpectedLogicError(String),
}

/// A wrapper for [`CPLErr::Type`] that reflects it as an enum
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub enum CplErrType {
    None = 0,
    Debug = 1,
    Warning = 2,
    Failure = 3,
    Fatal = 4,
}

impl From<CPLErr::Type> for CplErrType {
    fn from(error_type: CPLErr::Type) -> Self {
        if error_type > 4 {
            return Self::None; // fallback type, should not happen
        }

        unsafe { std::mem::transmute(error_type) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_that_gdal_error_is_send() {
        fn is_send<T: Send>() {
            // https://github.com/rust-lang/rust-clippy/issues/10318
            let _: [T; 0] = [];
        }

        is_send::<GdalError>();
    }
}
