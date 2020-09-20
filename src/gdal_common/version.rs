use crate::utils::_string;
use std::ffi::CString;

pub fn version_info(key: &str) -> String {
    let c_key = CString::new(key.as_bytes()).unwrap();
    unsafe { _string(gdal_sys::GDALVersionInfo(c_key.as_ptr())) }
}

#[cfg(test)]
mod tests {
    use super::version_info;

    #[test]
    fn test_version_info() {
        let release_date = version_info("RELEASE_DATE");
        let release_name = version_info("RELEASE_NAME");
        let version_text = version_info("--version");

        let mut date_iter = release_date.chars();

        let expected_text: String = format!(
            "GDAL {}, released {}/{}/{}",
            release_name,
            date_iter.by_ref().take(4).collect::<String>(),
            date_iter.by_ref().take(2).collect::<String>(),
            date_iter.by_ref().take(2).collect::<String>(),
        );

        assert_eq!(version_text, expected_text);
    }
}
