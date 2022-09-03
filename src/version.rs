//! GDAL Version Inspection Utilities
//!
//! ## Example
//!
//! Get the same string provided by using `--version` with the various GDAL CLI tools,
//! use [`VersionInfo::VERSION_SUMMARY`]:
//!
//! ```rust, no_run
//! use gdal::version::VersionInfo;
//! let gdal_ver = VersionInfo::VERSION_SUMMARY;
//! println!("{gdal_ver}")
//! ```
//! ```text,
//! GDAL 3.5.1, released 2022/06/30
//! ```

use crate::utils::_string;
use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};

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

/// Convenience selector for the various properties of GDAL version information that may be queried.
///
/// `VersionInfo` has a `Display` implementation which fetches the associated value from GDAL
/// and returns it as a string.
///
/// ## Example
///
/// For the string returned from passing `--version` to GDAL CLI tools,
/// use [`VERSION_SUMMARY`](VersionInfo::VERSION_SUMMARY):
///
/// ```rust, no_run
/// # use gdal::version::VersionInfo;
/// println!("{}", VersionInfo::VERSION_SUMMARY);
/// ```
/// ```text,
/// GDAL 3.5.1, released 2022/06/30
/// ```
/// For all the available version properties (except [`LICENSE`](VersionInfo::LICENSE)),
/// use [`VERSION_REPORT`](VersionInfo::VERSION_REPORT):
///
/// ```rust, no_run
/// # use gdal::version::VersionInfo;
/// let report = VersionInfo::VERSION_REPORT.to_string();
/// println!("{report}");
/// ```
/// ```text
/// GDALVersionInfo {
///     RELEASE_NAME: "3.5.1",
///     RELEASE_DATE: "20220630",
///     VERSION_NUM: "3050100",
///     BUILD_INFO:  {
///         PAM_ENABLED: "YES",
///         OGR_ENABLED: "YES",
///         GEOS_ENABLED: "YES",
///         GEOS_VERSION: "3.11.0-CAPI-1.17.0",
///         PROJ_BUILD_VERSION: "9.0.1",
///         PROJ_RUNTIME_VERSION: "9.0.1",
///     },
/// }
/// ```
#[allow(non_camel_case_types)]
#[non_exhaustive]
#[derive(Copy, Clone)]
pub enum VersionInfo {
    /// Returns one line version message suitable for use in response to version requests. i.e. “GDAL 1.1.7, released 2002/04/16”
    VERSION_SUMMARY,
    /// Returns GDAL_VERSION_NUM formatted as a string. i.e. “1170”
    VERSION_NUM,
    /// Returns GDAL_RELEASE_DATE formatted as a string. i.e. “20020416"
    RELEASE_DATE,
    /// Returns the GDAL_RELEASE_NAME. ie. “1.1.7”
    RELEASE_NAME,
    /// Returns the content of the LICENSE.TXT file from the GDAL_DATA directory.
    LICENSE,
    /// List of NAME=VALUE pairs separated by newlines with information on build time options.
    BUILD_INFO,
    /// Render all available version and build details in a multiline, debug string
    VERSION_REPORT,
}

use VersionInfo::*;
impl VersionInfo {
    /// Get the complete list of variants.
    pub fn options() -> Vec<Self> {
        vec![
            VERSION_SUMMARY,
            VERSION_NUM,
            RELEASE_DATE,
            RELEASE_NAME,
            LICENSE,
            BUILD_INFO,
            VERSION_REPORT,
        ]
    }
    /// Get the variant's name
    pub fn name(&self) -> &'static str {
        match self {
            VERSION_SUMMARY => "VERSION_SUMMARY",
            VERSION_NUM => "VERSION_NUM",
            RELEASE_DATE => "RELEASE_DATE",
            RELEASE_NAME => "RELEASE_NAME",
            LICENSE => "LICENSE",
            BUILD_INFO => "BUILD_INFO",
            VERSION_REPORT => "VERSION_REPORT",
        }
    }

    /// Fetch the key accepted by `version_info` if the variant has one.
    fn gdal_key(&self) -> Option<&'static str> {
        match self {
            VERSION_SUMMARY => Some("--version"),
            VERSION_NUM | RELEASE_DATE | RELEASE_NAME | LICENSE | BUILD_INFO => Some(self.name()),
            VERSION_REPORT => None,
        }
    }
}

/// Provides renderings of each variant name along with value provided by GDAL.
impl Debug for VersionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VERSION_REPORT => f
                .debug_struct("GDALVersionInfo")
                .field(RELEASE_NAME.name(), &RELEASE_NAME.to_string())
                .field(RELEASE_DATE.name(), &RELEASE_DATE.to_string())
                .field(VERSION_NUM.name(), &VERSION_NUM.to_string())
                .field(BUILD_INFO.name(), &BUILD_INFO)
                .finish(),
            BUILD_INFO => {
                // For uniform formatting, we parse the result from GDAL, which claims to be structured.
                let mut builder = f.debug_struct("");
                let text = BUILD_INFO.to_string();

                text.lines()
                    .filter_map(|l| l.split_once('='))
                    .for_each(|(key, value)| {
                        builder.field(key, &value);
                    });

                builder.finish()
            }
            i => f.debug_tuple(i.name()).field(&i.to_string()).finish(),
        }
    }
}

/// Fetches and formats the GDAL version property associated with each variant.
impl Display for VersionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(key) = self.gdal_key() {
            f.write_str(&version_info(key))
        } else {
            match self {
                VERSION_REPORT => f.write_fmt(format_args!("{self:#?}")),
                _ => unreachable!(
                    "{} should have `gdal_key` or be `VERSION_REPORT`",
                    self.name()
                ),
            }
        }
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        VERSION_SUMMARY
    }
}

#[cfg(test)]
mod tests {
    use super::version_info;
    use super::VersionInfo::*;

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
    fn test_version_enum() {
        let rel_name = RELEASE_NAME.to_string();
        assert!(!rel_name.is_empty());
        let rpt = VERSION_REPORT.to_string();
        assert!(rpt.contains(&rel_name));
    }
}
