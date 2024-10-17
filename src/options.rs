use std::ffi::c_uint;

use bitflags::bitflags;
use gdal_sys::GDALAccess;

/// Open options for [`crate::Dataset`]
#[derive(Debug, Default)]
pub struct DatasetOptions<'a> {
    pub open_flags: GdalOpenFlags,
    pub allowed_drivers: Option<&'a [&'a str]>,
    pub open_options: Option<&'a [&'a str]>,
    pub sibling_files: Option<&'a [&'a str]>,
}

// These are skipped by bindgen and manually updated.
bitflags! {
    /// GDal extended open flags used by [`Dataset::open_ex`].
    ///
    /// Used in the `nOpenFlags` argument to [`GDALOpenEx`].
    ///
    /// Note that the `GDAL_OF_SHARED` option is removed
    /// from the set of allowed option because it subverts
    /// the [`Send`] implementation that allow passing the
    /// dataset the another thread. See
    /// https://github.com/georust/gdal/issues/154.
    ///
    /// [`GDALOpenEx`]: https://gdal.org/doxygen/gdal_8h.html#a9cb8585d0b3c16726b08e25bcc94274a
    #[derive(Debug)]
    #[allow(clippy::assign_op_pattern)]
    pub struct GdalOpenFlags: c_uint {
        /// Open in read-only mode (default).
        const GDAL_OF_READONLY = 0x00;
        /// Open in update mode.
        const GDAL_OF_UPDATE = 0x01;
        /// Allow raster and vector drivers to be used.
        const GDAL_OF_ALL = 0x00;
        /// Allow raster drivers to be used.
        const GDAL_OF_RASTER = 0x02;
        /// Allow vector drivers to be used.
        const GDAL_OF_VECTOR = 0x04;
        /// Allow gnm drivers to be used.
        const GDAL_OF_GNM = 0x08;
        /// Allow multidimensional raster drivers to be used.
        #[cfg(all(major_ge_3,minor_ge_1))]
        const GDAL_OF_MULTIDIM_RASTER = 0x10;
        /// Emit error message in case of failed open.
        const GDAL_OF_VERBOSE_ERROR = 0x40;
        /// Open as internal dataset. Such dataset isn't
        /// registered in the global list of opened dataset.
        /// Cannot be used with GDAL_OF_SHARED.
        const GDAL_OF_INTERNAL = 0x80;

        /// Default strategy for cached blocks.
        const GDAL_OF_DEFAULT_BLOCK_ACCESS = 0;

        /// Array based strategy for cached blocks.
        const GDAL_OF_ARRAY_BLOCK_ACCESS = 0x100;

        /// Hashset based strategy for cached blocks.
        const GDAL_OF_HASHSET_BLOCK_ACCESS = 0x200;
    }
}

impl Default for GdalOpenFlags {
    fn default() -> GdalOpenFlags {
        GdalOpenFlags::GDAL_OF_READONLY
    }
}

impl From<GDALAccess::Type> for GdalOpenFlags {
    fn from(val: GDALAccess::Type) -> GdalOpenFlags {
        if val == GDALAccess::GA_Update {
            GdalOpenFlags::GDAL_OF_UPDATE
        } else {
            GdalOpenFlags::GDAL_OF_READONLY
        }
    }
}
