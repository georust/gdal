use libc::c_char;

#[allow(dead_code)]
#[allow(non_camel_case_types)]

extern {
    /// Set a configuration option for GDAL/OGR use
    pub fn CPLSetConfigOption(pszKey: *const c_char, pszValue: *const c_char);
    /// Get the value of a configuration option
    pub fn CPLGetConfigOption(pszKey: *const c_char, pszDefault: *const c_char) -> *const c_char;
}
