use crate::dataset::Dataset;
use crate::utils::_last_cpl_err;
use crate::vector::Geometry;
use gdal_sys::{self, CPLErr};
use libc::{c_char, c_void};
use std::{convert::TryFrom, ffi::CString, ptr};

use crate::errors::*;

#[derive(Copy, Clone, Debug)]
pub enum BurnSource {
    /// Use whatever `burn_values` argument is supplied to
    /// `rasterize`.
    UserSupplied,

    /// Add the geometry's Z value to whatever `burn_values` argument
    /// is supplied to `rasterize`.
    Z,
    // `M` is defined but seemingly not allowed for rasterization
}

#[derive(Copy, Clone, Debug)]
pub enum MergeAlgorithm {
    Replace,
    Add,
}

#[derive(Copy, Clone, Debug)]
pub enum OptimizeMode {
    Automatic,
    Raster,
    Vector,
}

/// Options that specify how to rasterize geometries.
#[derive(Copy, Clone, Debug)]
pub struct RasterizeOptions {
    /// Set to `true` to set all pixels touched by the line or
    /// polygons, not just those whose center is within the polygon or
    /// that are selected by brezenhams line algorithm. Defaults to
    /// `false`.
    pub all_touched: bool,

    /// May be set to `BurnSource::Z` to use the Z values of the
    /// geometries. `burn_value` is added to this before
    /// burning. Defaults to `BurnSource::UserSupplied` in which case
    /// just the `burn_value` is burned. This is implemented only for
    /// points and lines for now. `BurnValue::M` may be supported in
    /// the future.
    pub source: BurnSource,

    /// May be `MergeAlgorithm::Replace` (the default) or
    /// `MergeAlgorithm::Add`. `Replace` results in overwriting of
    /// value, while `Add` adds the new value to the existing raster,
    /// suitable for heatmaps for instance.
    pub merge_algorithm: MergeAlgorithm,

    /// The height in lines of the chunk to operate on. The larger the
    /// chunk size the less times we need to make a pass through all
    /// the shapes. If it is not set or set to zero the default chunk
    /// size will be used. Default size will be estimated based on the
    /// GDAL cache buffer size using formula: `cache_size_bytes /
    /// scanline_size_bytes`, so the chunk will not exceed the
    /// cache. Not used in `OPTIM=RASTER` mode.
    pub chunk_y_size: usize,

    pub optimize: OptimizeMode,
}

impl Default for RasterizeOptions {
    fn default() -> Self {
        RasterizeOptions {
            all_touched: false,
            source: BurnSource::UserSupplied,
            merge_algorithm: MergeAlgorithm::Replace,
            chunk_y_size: 0,
            optimize: OptimizeMode::Automatic,
        }
    }
}

/// An internal wrapper to simplify constructing **papszOptions. The
/// key is that you keep TextOptions around during a call that uses
/// its `as_ptr` result.
struct TextOptions {
    cstrings: Vec<CString>,
    ptrs: Vec<*const i8>,
}

impl TextOptions {
    fn new(strings: &[&str]) -> Result<TextOptions> {
        let cstrings: Result<Vec<CString>> = strings
            .iter()
            .map(|&s| CString::new(s).map_err(|error| error.into()))
            .collect();
        let cstrings = cstrings?;
        let ptrs = cstrings
            .iter()
            .map(|s| s.as_ptr())
            .chain(std::iter::once(ptr::null()))
            .collect();
        Ok(TextOptions { cstrings, ptrs })
    }

    fn as_ptr(&self) -> *const *const c_char {
        if self.cstrings.is_empty() {
            ptr::null()
        } else {
            self.ptrs.as_ptr()
        }
    }
}

impl TryFrom<&RasterizeOptions> for TextOptions {
    type Error = GdalError;

    fn try_from(options: &RasterizeOptions) -> Result<Self> {
        let mut strings = vec![
            format!(
                "ALL_TOUCHED={}",
                if options.all_touched { "TRUE" } else { "FALSE" }
            ),
            format!(
                "MERGE_ALG={}",
                match options.merge_algorithm {
                    MergeAlgorithm::Replace => "REPLACE",
                    MergeAlgorithm::Add => "ADD",
                }
            ),
            format!("CHUNKYSIZE={}", options.chunk_y_size),
            format!(
                "OPTIM={}",
                match options.optimize {
                    OptimizeMode::Automatic => "AUTO",
                    OptimizeMode::Raster => "RASTER",
                    OptimizeMode::Vector => "VECTOR",
                }
            ),
        ];
        if let BurnSource::Z = options.source {
            strings.push("BURN_VALUE_FROM=Z".to_string());
        }

        let strs: Vec<&str> = strings.iter().map(String::as_str).collect();
        TextOptions::new(&strs)
    }
}

/// Burn geometries into raster.
///
/// Rasterize a sequence of `gdal::vector::Geometry` onto some
/// `dataset` bands. Those geometries must have coordinates
/// georegerenced to `dataset`.
///
/// Bands are selected using indices supplied in `bands`.
///
/// Options are specified with `options`.
///
/// There must be one burn value for every geometry. The output raster
/// may be of any GDAL supported datatype.
pub fn rasterize(
    dataset: &mut Dataset,
    bands: &[isize],
    geometries: &[Geometry],
    burn_values: &[f64],
    options: Option<RasterizeOptions>,
) -> Result<()> {
    assert!(!bands.is_empty());
    assert_eq!(burn_values.len(), geometries.len());

    let bands: Vec<i32> = bands.iter().map(|&band| band as i32).collect();
    let options = options.unwrap_or_default();

    let geometries: Vec<_> = geometries
        .iter()
        .map(|geo| unsafe { geo.c_geometry() })
        .collect();
    let burn_values: Vec<f64> = burn_values
        .iter()
        .flat_map(|burn| std::iter::repeat(burn).take(bands.len()))
        .copied()
        .collect();

    let text_options: TextOptions = TryFrom::try_from(&options)?;
    unsafe {
        // The C function takes `bands`, `geometries`, `burn_values`
        // and `options` without mention of `const`, and this is
        // propagated to the gdal_sys wrapper. The lack of `const`
        // seems like a mistake in the GDAL API, so we just do a casts
        // here.

        let error = gdal_sys::GDALRasterizeGeometries(
            dataset.c_dataset(),
            bands.len() as i32,
            bands.as_ptr() as *mut i32,
            geometries.len() as i32,
            geometries.as_ptr() as *mut *mut c_void,
            None,
            ptr::null_mut(),
            burn_values.as_ptr() as *mut f64,
            text_options.as_ptr() as *mut *mut i8,
            None,
            ptr::null_mut(),
        );
        if error != CPLErr::CE_None {
            return Err(_last_cpl_err(error));
        }
    }
    Ok(())
}
