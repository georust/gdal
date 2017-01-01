use libc::{c_int, c_char};

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
#[repr(C)]
pub enum CPLErr {
  CE_None = 0,
  CE_Debug = 1,
  CE_Warning = 2,
  CE_Failure = 3,
  CE_Fatal = 4 
}

#[link(name="gdal")]
extern {
    /// Erase any traces of previous errors.
    pub fn CPLErrorReset();

    /// Fetch the last error number.
    pub fn CPLGetLastErrorNo() -> c_int;
    
    /// Fetch the last error type.
    pub fn CPLGetLastErrorType() -> CPLErr;

    /// Get the last error message.    
    pub fn CPLGetLastErrorMsg() -> *const c_char;
}

