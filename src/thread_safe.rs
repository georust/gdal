use std::{ops::Deref, sync::Arc};

use crate::errors::DatasetNotThreadSafeError;
use crate::{Dataset, GdalOpenFlags};

impl Dataset {
    /// Return whether this dataset, and its related objects (typically raster
    /// bands), can be called for the intended scope.
    ///
    /// See [`gdal_sys::GDALDatasetIsThreadSafe`].
    ///
    /// Note: currently, `scope_flags` must be set to `GDAL_OF_RASTER`, as
    /// thread-safety is limited to read-only operations and excludes
    /// operations on vector layers or usage of the multidimensional API.
    pub fn is_thread_safe(&self, scope_flags: GdalOpenFlags) -> bool {
        unsafe {
            use std::ptr;

            gdal_sys::GDALDatasetIsThreadSafe(
                self.c_dataset(),
                scope_flags.bits() as i32,
                ptr::null_mut(),
            )
        }
    }

    /// Convert this dataset into a thread-safe dataset.
    ///
    /// Note: only read-only rasters are supported, and `scope_flags` must be set
    /// to `GDAL_OF_RASTER`. If the dataset is not thread-safe, the original one
    /// can be recovered from the error.
    ///
    /// # Example
    /// ```no_run
    /// # use gdal::{Dataset, DatasetOptions, GdalOpenFlags};
    /// let ds = Dataset::open_ex(
    ///     "file.tif",
    ///     DatasetOptions {
    ///         open_flags: GdalOpenFlags::GDAL_OF_RASTER | GdalOpenFlags::GDAL_OF_THREAD_SAFE,
    ///         ..Default::default()
    ///     },
    /// )?;
    ///
    /// match ds.try_into_thread_safe(GdalOpenFlags::GDAL_OF_RASTER) {
    ///     Ok(thread_safe_ds) => { /* use it */ },
    ///     Err(e) => {
    ///         let ds = e.into_inner(); // recover the original dataset
    ///         // ... handle non-thread-safe case ...
    ///     }
    /// }
    /// # Ok::<_, gdal::errors::GdalError>(())
    /// ```
    pub fn try_into_thread_safe(
        self,
        scope_flags: GdalOpenFlags,
    ) -> std::result::Result<ThreadSafeDataset, DatasetNotThreadSafeError> {
        if self.is_thread_safe(scope_flags) {
            // We consume the `Dataset` to prevent the user from closing it.
            //
            // SAFETY: `is_thread_safe` just said it's thread-safe.
            let dataset = unsafe { ThreadSafeDataset::new(self) };
            Ok(dataset)
        } else {
            let err = DatasetNotThreadSafeError(self);
            Err(err)
        }
    }
}

/// This is a wrapper type for passing a thread-safe [`Dataset`] across threads.
///
/// Only read-only rasters can be thread-safe.
///
/// Note: most drivers don't support native thread-safety, LIBRETIFF being a
/// notable exception. For the others, GDAL will reopen the dataset multiple
/// times. On Unix systems, you might want to bump the open files limit (for
/// example, see `ulimit` and the `rlimit` crate).
///
/// # Example
/// ```no_run
/// use gdal::{Dataset, DatasetOptions, GdalOpenFlags};
/// use std::thread;
///
/// let ds = Dataset::open_ex(
///     "file.tif",
///     DatasetOptions {
///         open_flags: GdalOpenFlags::GDAL_OF_RASTER | GdalOpenFlags::GDAL_OF_THREAD_SAFE,
///         ..Default::default()
///     },
/// )?;
/// let thread_safe_ds = ds.try_into_thread_safe(GdalOpenFlags::GDAL_OF_RASTER)?;
///
/// // Clone and use across threads
/// thread::scope(|s| {
///     for _ in 0..4 {
///         let ds = thread_safe_ds.clone();
///         s.spawn(move || {
///             let band = ds.rasterband(1)?;
///             // ... read from band ...
///             Ok::<_, gdal::errors::GdalError>(())
///         });
///     }
/// });
/// # Ok::<_, gdal::errors::GdalError>(())
/// ```
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ThreadSafeDataset {
    inner: Arc<Dataset>,
}

unsafe impl Sync for ThreadSafeDataset {}
unsafe impl Send for ThreadSafeDataset {}

impl ThreadSafeDataset {
    /// # Safety
    /// Dataset must be thread-safe.
    unsafe fn new(dataset: Dataset) -> Self {
        Self {
            // `GDALReferenceDataset` is not thread-safe, so we keep our own reference count using an `Arc`.
            #[allow(clippy::arc_with_non_send_sync)]
            inner: Arc::new(dataset),
        }
    }
}

impl AsRef<Dataset> for ThreadSafeDataset {
    fn as_ref(&self) -> &Dataset {
        &self.inner
    }
}

impl Deref for ThreadSafeDataset {
    type Target = Dataset;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::errors::{GdalError, Result};
    use crate::test_utils::fixture;
    use crate::{Dataset, DatasetOptions, GdalOpenFlags};

    #[test]
    fn test_thread_safe_dataset() -> Result<()> {
        let ds = Dataset::open_ex(
            fixture("tinymarble.tif"),
            DatasetOptions {
                open_flags: GdalOpenFlags::GDAL_OF_RASTER,
                allowed_drivers: None,
                open_options: None,
                sibling_files: None,
            },
        )?;
        assert!(!ds.is_thread_safe(GdalOpenFlags::empty()));
        assert!(!ds.is_thread_safe(GdalOpenFlags::GDAL_OF_RASTER));
        ds.try_into_thread_safe(GdalOpenFlags::GDAL_OF_RASTER)
            .unwrap_err()
            .into_inner();

        let ds = Dataset::open_ex(
            fixture("tinymarble.tif"),
            DatasetOptions {
                open_flags: GdalOpenFlags::GDAL_OF_RASTER | GdalOpenFlags::GDAL_OF_THREAD_SAFE,
                ..Default::default()
            },
        )?;
        assert!(!ds.is_thread_safe(GdalOpenFlags::empty()));
        assert!(ds.is_thread_safe(GdalOpenFlags::GDAL_OF_RASTER));

        let ds = ds.try_into_thread_safe(GdalOpenFlags::GDAL_OF_RASTER)?;
        thread::scope(|s| {
            let threads = (0..10)
                .map(|_| {
                    let ds = ds.clone();
                    s.spawn(move || {
                        let band = ds.rasterband(1)?;
                        let checksum = band.checksum((0, 0), band.size())?;
                        assert_eq!(checksum, 44419);
                        let band = ds.rasterband(2)?;
                        let checksum = band.checksum((0, 0), band.size())?;
                        assert_eq!(checksum, 55727);
                        let band = ds.rasterband(3)?;
                        let checksum = band.checksum((0, 0), band.size())?;
                        assert_eq!(checksum, 61385);
                        Ok::<_, GdalError>(())
                    })
                })
                .collect::<Vec<_>>();
            for thread in threads {
                thread.join().unwrap()?;
            }
            Ok(())
        })
    }
}
