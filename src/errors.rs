use libc::{c_int};
use gdal_sys::cpl_error::CPLErr;
use gdal_sys::ogr_enums::{OGRErr, OGRFieldType};

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
        NullPointer(method_name: &'static str, msg: String) {
            description("GDAL method returned a NULL pointer.")
            display("GDAL method '{}' returned a NULL pointer. Error msg: '{}'", method_name, msg)
        }
        OgrError(err: OGRErr, method_name: &'static str) {
            description("OGR error")
            display("OGR method '{}' returned error: '{:?}'", method_name, err)
        }
        UnhandledFieldType(field_type: OGRFieldType, method_name: &'static str){
            description("Unhandled field type")
            display("Unhandled type {:?} on OGR method {}", field_type, method_name)
        }
        InvalidFieldName(field_name: String, method_name: &'static str){
            description("Invalid field name error")
            display("Invalid field name '{}' used on method {}", field_name, method_name)
        }
        InvalidFieldIndex(index: usize, method_name: &'static str){
            description("Invalid field index error")
            display("Invalid field index {} used on method {}", index, method_name)
        }
        UnlinkedGeometry(method_name: &'static str){
            description("Unlinked Geometry")
            display("Unlinked Geometry on method {}", method_name)
        }
    }
}
