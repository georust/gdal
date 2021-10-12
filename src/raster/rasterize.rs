use crate::utils::_last_cpl_err;
use crate::vector::Geometry;
use crate::{dataset::Dataset, driver::CslStringList};
use gdal_sys::{self, CPLErr};
use libc::c_void;
use std::convert::TryFrom;
use std::ptr;

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

impl TryFrom<RasterizeOptions> for CslStringList {
    type Error = GdalError;

    fn try_from(value: RasterizeOptions) -> Result<CslStringList> {
        let mut options = CslStringList::new();

        options.set_name_value(
            "ALL_TOUCHED",
            if value.all_touched { "TRUE" } else { "FALSE" },
        )?;
        options.set_name_value(
            "MERGE_ALG",
            match value.merge_algorithm {
                MergeAlgorithm::Replace => "REPLACE",
                MergeAlgorithm::Add => "ADD",
            },
        )?;
        options.set_name_value("CHUNKYSIZE", &value.chunk_y_size.to_string())?;
        options.set_name_value(
            "OPTIM",
            match value.optimize {
                OptimizeMode::Automatic => "AUTO",
                OptimizeMode::Raster => "RASTER",
                OptimizeMode::Vector => "VECTOR",
            },
        )?;
        if let BurnSource::Z = value.source {
            options.set_name_value("BURN_VALUE_FROM", "Z")?;
        }

        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::CslStringList;

    use super::RasterizeOptions;
    use std::{
        convert::TryFrom,
        ffi::{CStr, CString},
    };

    fn fetch(c_options: &CslStringList, key: &str) -> Option<String> {
        let key = CString::new(key).unwrap();
        unsafe {
            let c_value = gdal_sys::CSLFetchNameValue(c_options.as_ptr(), key.as_ptr());
            if c_value.is_null() {
                None
            } else {
                Some(CStr::from_ptr(c_value).to_str().unwrap())
            }
        }
        .map(String::from)
    }

    #[test]
    fn test_rasterizeoptions_as_ptr() {
        let c_options = CslStringList::try_from(RasterizeOptions::default()).unwrap();
        assert_eq!(fetch(&c_options, "ALL_TOUCHED"), Some("FALSE".to_string()));
        assert_eq!(fetch(&c_options, "BURN_VALUE_FROM"), None);
        assert_eq!(fetch(&c_options, "MERGE_ALG"), Some("REPLACE".to_string()));
        assert_eq!(fetch(&c_options, "CHUNKYSIZE"), Some("0".to_string()));
        assert_eq!(fetch(&c_options, "OPTIM"), Some("AUTO".to_string()));
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
    if bands.is_empty() {
        return Err(GdalError::BadArgument(
            "`bands` must not be empty".to_string(),
        ));
    }
    if burn_values.len() != geometries.len() {
        return Err(GdalError::BadArgument(format!(
            "Burn values length ({}) must match geometries length ({})",
            burn_values.len(),
            geometries.len()
        )));
    }
    for band in bands {
        let is_good = *band > 0 && *band <= dataset.raster_count();
        if !is_good {
            return Err(GdalError::BadArgument(format!(
                "Band index {} is out of bounds",
                *band
            )));
        }
    }

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

    let c_options = CslStringList::try_from(options).unwrap().as_ptr();
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
            c_options as *mut *mut i8,
            None,
            ptr::null_mut(),
        );
        if error != CPLErr::CE_None {
            return Err(_last_cpl_err(error));
        }
    }
    Ok(())
}
