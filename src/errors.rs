use libc::{c_int};

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        FfiNulError(::std::ffi::NulError);
        StrUtf8Error(::std::str::Utf8Error);
    }

    errors {
        CplError(class: c_int, number: c_int, msg: String) {
            description("GDAL internal error")
            display("CPL error class: '{}', error number: '{}', error msg: '{}'", class, number, msg)
        }
        NullPointer(method_name: &'static str) {
            description("GDAL method returned a NULL pointer.")
            display("GDAL method {} returned a NULL pointer.", method_name) 
        }
    }
}
