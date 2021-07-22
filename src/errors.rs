use libc::c_int;
use thiserror::Error;

use gdal_sys::{CPLErr, OGRErr, OGRFieldType, OGRwkbGeometryType};

pub type Result<T> = std::result::Result<T, GdalError>;

#[derive(Clone, PartialEq, Debug, Error)]
pub enum GdalError {
    #[error("FfiNulError")]
    FfiNulError(#[from] std::ffi::NulError),
    #[error("FfiIntoStringError")]
    FfiIntoStringError(#[from] std::ffi::IntoStringError),
    #[error("StrUtf8Error")]
    StrUtf8Error(#[from] std::str::Utf8Error),
    #[cfg(feature = "ndarray")]
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
}
