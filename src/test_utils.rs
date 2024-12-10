use crate::vsi::unlink_mem_file;
use crate::{Dataset, DatasetOptions};
use gdal_sys::GDALAccess;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use tempfile::TempPath;

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
pub fn fixture(filename: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(filename)
}

/// A struct that represents a `/vsimem/` (in-memory) path.
///
/// The file will be deleted when the value is dropped.
pub struct InMemoryFixture {
    path: PathBuf,
}

impl InMemoryFixture {
    pub fn new(filename: &str) -> Self {
        // TODO: use `VSIStatL` to make sure the file doesn't exist.
        let mut path = PathBuf::from("/vsimem");
        path.push(filename);

        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for InMemoryFixture {
    fn drop(&mut self) {
        unlink_mem_file(&self.path).expect("unable to remove in-memory file");
    }
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

/// Copies the given file to a temporary file and opens it for writing. When the returned
/// `TempPath` is dropped, the file is deleted.
pub fn open_gpkg_for_update(path: &Path) -> (TempPath, Dataset) {
    use std::fs;
    use std::io::Write;

    let input_data = fs::read(path).unwrap();
    let (mut file, temp_path) = tempfile::Builder::new()
        .suffix(".gpkg")
        .tempfile()
        .unwrap()
        .into_parts();
    file.write_all(&input_data).unwrap();
    // Close the temporary file so that Dataset can open it safely even if the filesystem uses
    // exclusive locking (Windows?).
    drop(file);

    let ds = Dataset::open_ex(
        &temp_path,
        DatasetOptions {
            open_flags: GDALAccess::GA_Update.into(),
            allowed_drivers: Some(&["GPKG"]),
            ..DatasetOptions::default()
        },
    )
    .unwrap();
    (temp_path, ds)
}

/// Copies the given file to a temporary file and opens it for writing. When the returned
/// `TempPath` is dropped, the file is deleted.
pub fn open_dataset_for_update(path: &Path) -> (TempPath, Dataset) {
    use std::fs;
    use std::io::Write;

    let input_data = fs::read(path).unwrap();
    let (mut file, temp_path) = tempfile::Builder::new()
        // using the whole filename as suffix should be fine (can't
        // use .extension() for .shp.zip and such)
        .suffix(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
        )
        .tempfile()
        .unwrap()
        .into_parts();
    file.write_all(&input_data).unwrap();
    // Close the temporary file so that Dataset can open it safely even if the filesystem uses
    // exclusive locking (Windows?).
    drop(file);

    let ds = Dataset::open_ex(
        &temp_path,
        DatasetOptions {
            open_flags: GDALAccess::GA_Update.into(),
            ..DatasetOptions::default()
        },
    )
    .unwrap();
    (temp_path, ds)
}

/// Assert numerical difference between two expressions is less than
/// 64-bit machine epsilon or a specified epsilon.
///
/// # Examples:
/// ```rust, no_run
/// use gdal::assert_near;
/// use std::f64::consts::{PI, E};
/// assert_near!(PI / E, 1.1557273497909217);
/// // with specified epsilon
/// assert_near!(PI / E, 1.15572734, epsilon = 1e-8);
/// ```
#[macro_export]
macro_rules! assert_near {
    ($left:expr, $right:expr) => {
        assert_near!($left, $right, epsilon = f64::EPSILON)
    };
    ($left:expr, $right:expr, epsilon = $ep:expr) => {
        assert!(
            ($left - $right).abs() < $ep,
            "|{} - {}| = {} is greater than epsilon {:.4e}",
            $left,
            $right,
            ($left - $right).abs(),
            $ep
        )
    };
    ($left:expr, $right:expr, epsilon = $ep:expr, field = $field:expr) => {
        assert!(
            ($left - $right).abs() < $ep,
            "field {}: |{} - {}| = {} is greater than epsilon {:.4e}",
            $field,
            $left,
            $right,
            ($left - $right).abs(),
            $ep
        )
    };
    // Pseudo-specialization
    (StatisticsAll, $left:expr, $right:expr, epsilon = $ep:expr) => {
        assert_near!($left.min, $right.min, epsilon = $ep, field = "min");
        assert_near!($left.max, $right.max, epsilon = $ep, field = "max");
        assert_near!($left.mean, $right.mean, epsilon = $ep, field = "mean");
        assert_near!(
            $left.std_dev,
            $right.std_dev,
            epsilon = $ep,
            field = "std_dev"
        );
    };
}
