use libc::{c_int};
use std::{self, fmt, result};

use gdal_sys::{CPLErr, OGRErr, OGRFieldType};
use failure::{Context, Fail, Backtrace};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, PartialEq, Debug, Fail)]
pub enum ErrorKind {

    #[fail(display = "FfiNulError")]
    FfiNulError(#[cause] std::ffi::NulError),
    #[fail(display = "StrUtf8Error")]
    StrUtf8Error(#[cause] std::str::Utf8Error),
    #[cfg(feature = "ndarray")]
    #[fail(display = "NdarrayShapeError")]
    NdarrayShapeError(#[cause] ndarray::ShapeError),
    #[fail(display = "CPL error class: '{:?}', error number: '{}', error msg: '{}'", class, number, msg)]
    CplError {
        class: CPLErr::Type,
        number: c_int,
        msg: String
    },
    #[fail(display ="GDAL method '{}' returned a NULL pointer. Error msg: '{}'", method_name, msg)]
    NullPointer {
        method_name: &'static str,
        msg: String
    },
    #[fail(display = "Can't cast to f64")]
    CastToF64Error,
    #[fail(display ="OGR method '{}' returned error: '{:?}'", method_name, err)]
    OgrError {
        err: OGRErr::Type,
        method_name: &'static str
    },
    #[fail(display ="Unhandled type {:?} on OGR method {}", field_type, method_name)]
    UnhandledFieldType {
        field_type: OGRFieldType::Type,
        method_name: &'static str
    },
    #[fail(display ="Invalid field name '{}' used on method {}", field_name, method_name)]
    InvalidFieldName {
        field_name: String,
        method_name: &'static str
    },
    #[fail(display ="Invalid field index {} used on method {}", index, method_name)]
    InvalidFieldIndex {
        index: usize,
        method_name: &'static str
    },
    #[fail(display ="Unlinked Geometry on method {}", method_name)]
    UnlinkedGeometry {
        method_name: &'static str
    },
    #[fail(display ="Invalid coordinate range while transforming points from {} to {}: {:?}", from, to, msg)]
    InvalidCoordinateRange {
        from: String,
        to: String,
        msg: Option<String>
    }

}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn kind_ref(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Context::new(kind) }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Error {
        Error { inner: Context::new(ErrorKind::FfiNulError(err)) }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Error {
        Error { inner: Context::new(ErrorKind::StrUtf8Error(err)) }
    }
}

#[cfg(feature = "ndarray")]
impl From<ndarray::ShapeError> for Error {
    fn from(err: ndarray::ShapeError) -> Error {
        Error { inner: Context::new(ErrorKind::NdarrayShapeError(err)) }
    }
}
