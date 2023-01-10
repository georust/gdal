use std::ffi::c_void;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// A struct that contains a temporary directory and a path to a file in that directory.
pub struct TempFixture {
    _temp_dir: tempfile::TempDir,
    temp_path: PathBuf,
}

impl TempFixture {
    /// Creates a copy of the test file in a temporary directory.
    /// Returns the struct `TempFixture` that contains the temp dir (for clean-up on `drop`) as well as the path to the file.
    ///
    /// This can potentially be removed when <https://github.com/OSGeo/gdal/issues/6253> is resolved.
    pub fn fixture(name: &str) -> Self {
        let staging = Self::empty(name);
        let source = Path::new("fixtures").join(name);
        std::fs::copy(source, &staging.temp_path).unwrap();
        staging
    }

    /// Creates a temporary directory and path to a non-existent file with given `name`.
    /// Useful for writing results to during testing
    ///
    /// Returns the struct `TempFixture` that contains the temp dir (for clean-up on `drop`)
    /// as well as the empty file path.
    pub fn empty(name: &str) -> Self {
        let _temp_dir = tempfile::tempdir().unwrap();
        let temp_path = _temp_dir.path().join(name);
        Self {
            _temp_dir,
            temp_path,
        }
    }

    pub fn path(&self) -> &Path {
        &self.temp_path
    }
}

impl AsRef<Path> for TempFixture {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

/// Returns the fully qualified path to `filename` in `${CARGO_MANIFEST_DIR}/fixtures`.
pub(crate) fn fixture(filename: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(filename)
}

/// Scoped value for temporarily suppressing thread-local GDAL log messages.
///
/// Useful for tests that expect GDAL errors and want to keep the output log clean
/// of distracting yet expected error messages.
pub(crate) struct SuppressGDALErrorLog {
    // Make !Sync and !Send, and force use of `new`.
    _private: PhantomData<*mut c_void>,
}

impl SuppressGDALErrorLog {
    pub(crate) fn new() -> Self {
        unsafe { gdal_sys::CPLPushErrorHandler(Some(gdal_sys::CPLQuietErrorHandler)) };
        SuppressGDALErrorLog {
            _private: PhantomData,
        }
    }
}

impl Drop for SuppressGDALErrorLog {
    fn drop(&mut self) {
        unsafe { gdal_sys::CPLPopErrorHandler() };
    }
}
