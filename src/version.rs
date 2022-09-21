//! GDAL Version Inspection Utilities
//!
//! ## Example
//!
//! Get the same string provided by using `--version` with the various GDAL CLI tools,
//! use [`VersionInfo::version_summary`]:
//!
//! ```rust, no_run
//! use gdal::version::VersionInfo;
//! let gdal_ver = VersionInfo::version_summary();
//! println!("{gdal_ver}")
//! ```
//! ```text,
//! GDAL 3.5.1, released 2022/06/30
//! ```

use crate::utils::_string;
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt::Write;

/// Calls `GDALVersionInfo`, expecting `key` as one of the following values:
///
/// “VERSION_NUM”, “RELEASE_DATE”, “RELEASE_NAME”, "-–version”, “LICENSE”, “BUILD_INFO”.
///
/// See [`VersionInfo`] for a more ergonomic means of accessing these components.
///
/// Details: [`const char *GDALVersionInfo(const char*)`](https://gdal.org/api/raster_c_api.html#_CPPv415GDALVersionInfoPKc)
pub fn version_info(key: &str) -> String {
    let c_key = CString::new(key.as_bytes()).unwrap();
    _string(unsafe { gdal_sys::GDALVersionInfo(c_key.as_ptr()) })
}

/// Convenience functions for the various pre-defined queryable properties of GDAL version information.
///
/// ## Example
///
/// For the string returned from passing `--version` to GDAL CLI tools,
/// use [`VersionInfo::version_summary`]:
///
/// ```rust, no_run
/// # use gdal::version::VersionInfo;
/// println!("{}", VersionInfo::version_summary());
/// ```
/// ```text,
/// GDAL 3.5.1, released 2022/06/30
/// ```
/// For all the available version properties (except [`VersionInfo::license`],
/// use [VersionInfo::version_report]:
///
/// ```rust, no_run
/// # use gdal::version::VersionInfo;
/// let report = VersionInfo::version_report();
/// println!("{report}");
/// ```
/// ```text
/// GDALVersionInfo {
///     RELEASE_NAME: "3.5.1"
///     RELEASE_DATE: "20220630"
///     VERSION_NUM: "3050100"
///     BUILD_INFO {
///         PAM_ENABLED: "YES"
///         PROJ_BUILD_VERSION: "9.0.1"
///         OGR_ENABLED: "YES"
///         PROJ_RUNTIME_VERSION: "9.0.1"
///         GEOS_ENABLED: "YES"
///         GEOS_VERSION: "3.11.0-CAPI-1.17.0"
///     }
/// }
/// ```
pub struct VersionInfo;
impl VersionInfo {
    /// Returns one line version message suitable for use in response to version requests. i.e. “GDAL 1.1.7, released 2002/04/16”
    pub fn version_summary() -> String {
        version_info("--version")
    }
    /// Returns GDAL_VERSION_NUM formatted as a string. i.e. “1170”
    pub fn version_num() -> String {
        version_info("VERSION_NUM")
    }
    /// Returns GDAL_RELEASE_DATE formatted as a string. i.e. “20020416"
    pub fn release_date() -> String {
        version_info("RELEASE_DATE")
    }
    /// Returns the GDAL_RELEASE_NAME. ie. “1.1.7”
    pub fn release_name() -> String {
        version_info("RELEASE_NAME")
    }
    /// Returns the content of the LICENSE.TXT file from the GDAL_DATA directory.
    pub fn license() -> String {
        version_info("LICENSE")
    }
    /// Get a dictionary of GDAL build configuration options, such as `GEOS_VERSION` and
    /// `OGR_ENABLED`.
    pub fn build_info() -> HashMap<String, String> {
        let text = version_info("BUILD_INFO");
        text.lines()
            .filter_map(|l| l.split_once('='))
            .map(|p| (p.0.to_string(), p.1.to_string()))
            .collect()
    }
    /// Render all available version and build details in a multiline, debug string
    pub fn version_report() -> String {
        let mut buff: String = "GDALVersionInfo {\n".into();

        fn kv(buff: &mut String, l: usize, k: &str, v: &str) {
            writeln!(buff, "{:indent$}{k}: \"{v}\"", " ", indent = l * 4).unwrap();
        }

        kv(&mut buff, 1, "RELEASE_NAME", &Self::release_name());
        kv(&mut buff, 1, "RELEASE_DATE", &Self::release_date());
        kv(&mut buff, 1, "VERSION_NUM", &Self::version_num());
        buff.push_str("    BUILD_INFO {\n");

        Self::build_info()
            .iter()
            .for_each(|(k, v)| kv(&mut buff, 2, k, v));

        buff.push_str("    }\n");
        buff.push('}');
        buff
    }
}

#[cfg(test)]
mod tests {
    use super::version_info;
    use crate::version::VersionInfo;

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

    #[test]
    fn test_version_info_functions() {
        let rel_name = VersionInfo::release_name();
        assert!(!rel_name.is_empty());

        let build = VersionInfo::build_info();
        assert!(!build.is_empty());

        let rpt = VersionInfo::version_report();
        assert!(rpt.contains(&rel_name));

        let license = VersionInfo::license();
        assert!(!license.is_empty());
    }
}
