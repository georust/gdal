use libc::{c_int};
use gdal_sys::cpl_error::CPLErr;
use gdal_sys::ogr_enums::OGRErr;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        FfiNulError(::std::ffi::NulError);
        StrUtf8Error(::std::str::Utf8Error);
    }

    errors {
        CplError(class: CPLErr, number: c_int, msg: String) {
            description("GDAL internal error")
            display("CPL error class: '{:?}', error number: '{}', error msg: '{}'", class, number, msg)
        }
        NullPointer(method_name: &'static str) {
            description("GDAL method returned a NULL pointer.")
            display("GDAL method '{}' returned a NULL pointer.", method_name)
        }
        OgrError(err: OGRErr, method_name: &'static str) {
            description("OGR error")
            display("OGR method '{}' returned error: '{:?}'", method_name, err)
        }
        InvalidInput(method_name: &'static str) {
            description("Invalid input")
            display("Invalid input : {}", method_name)
        }
    }
}
