//! Digital Elevation Model (DEM) processing routines.
//!
//! This module provides bindings to the algorithms in the
//! [`gdaldem` tool](https://gdal.org/programs/gdaldem.html#gdaldem).
//!
//! The routines assume an open dataset containing customary digital elevation model data.
//! This includes assumptions that `x` (east-west), `y` (north-south), and `z` (elevation) units are identical.
//! If `x` and `y` units are identical, but `z` (elevation) units are different,
//! `hillshade` and `slope` support a scale setting to set the ratio of vertical units to horizontal.
//! See [`SlopeOptions::with_scale`] for details.
//!
//! # Examples
//!
//! Examples may be found associated with the following functions:
//!
//! * [`aspect()`]
//! * [`color_relief()`]
//! * [`hillshade()`]
//! * [`roughness()`]
//! * [`slope()`]
//! * [`terrain_ruggedness_index()`]
//! * [`topographic_position_index()`]
//!

#![deny(missing_docs)]

use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;

use libc::c_int;

pub use aspect::*;
pub use color_relief::*;
use gdal_sys::{CPLErr, GDALDEMProcessing};
pub use hillshade::*;
pub use options::{DemAlg, DemSlopeAlg};
pub use roughness::*;
pub use slope::*;
pub use tpi::*;
pub use tri::*;

use crate::cpl::CslStringList;
use crate::errors::Result;
use crate::utils::{_last_cpl_err, _path_to_c_string};
use crate::Dataset;

mod aspect;
mod color_relief;
mod hillshade;
mod options;
mod roughness;
mod slope;
mod tpi;
mod tri;

/// Slope aspect-angle routine for DEM datasets.
///
/// This method outputs a 32-bit float raster with values between 0° and 360°
/// representing the azimuth that slopes are facing. The definition of the azimuth is such that:
///
/// * 0° means that the slope is facing the North,
/// * 90° it's facing the East,
/// * 180° it's facing the South and;
/// * 270° it's facing the West (provided that the top of your input raster is north oriented).
///
/// By default, the aspect value `-9999` is used as the no-data value to indicate undefined aspect in flat
/// areas with slope=0. See [`AspectOptions::with_zero_for_flat`] for alternative.
///
/// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
///
/// # Example
///
/// ```rust, no_run
/// use gdal::Dataset;
/// # fn main() -> gdal::errors::Result<()> {
/// use std::path::Path;
/// use gdal::raster::dem::*;
/// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
/// let mut opts = AspectOptions::new();
/// opts
///     .with_algorithm(DemSlopeAlg::Horn)
///     .with_zero_for_flat(true);
/// let aspect_ds = aspect(&ds, Path::new("target/dem-hills-aspect.tiff"), &opts)?;
/// let stats = aspect_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
/// println!("{stats:#?}");
/// # Ok(())
/// # }
/// ```
/// The resulting output is:
///
/// ```text
/// StatisticsAll {
///     min: 0.0,
///     max: 359.9951171875,
///     mean: 165.72752499997543,
///     std_dev: 98.5901999514453,
/// }
/// ```
///
/// See: [`gdaldem aspect`](https://gdal.org/programs/gdaldem.html#aspect) for details,
pub fn aspect<P: AsRef<Path>>(
    ds: &Dataset,
    dest_file: P,
    options: &AspectOptions,
) -> Result<Dataset> {
    dem_eval(
        ds,
        dest_file.as_ref(),
        DemAlg::Aspect,
        &options.to_options_list()?,
        None,
    )
}

/// Generate a color-relief rendering of DEM data.
///
/// This routine outputs a 3-band (RGB) or 4-band (RGBA) raster with values computed from
/// the elevation and a text-based color configuration file.
///
/// The color configuration file contains associations between various elevation values
/// and the corresponding desired color. See [`ColorReliefOptions::new`] for details.
///
/// By default, the colors between the given elevation
/// values are blended smoothly and the result is a nice colorized DEM.
/// The [`ColorMatchingMode::ExactColorEntry`] or [`ColorMatchingMode::NearestColorEntry`] options
/// can be used to avoid that linear interpolation for values that don't match an index of
/// the color configuration file. See [`ColorMatchingMode`] for details.
///
/// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
///
/// # Example
///
/// ```rust, no_run
/// use gdal::Dataset;
/// # fn main() -> gdal::errors::Result<()> {
/// use std::path::Path;
/// use gdal::raster::dem::*;
/// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
/// let mut opts = ColorReliefOptions::new("fixtures/color-relief.clr");
/// opts.with_alpha(true);
/// let hs_ds = color_relief(&ds, Path::new("target/dem-hills-relief.tiff"), &opts)?;
/// // Note: Output will actually be a 4-band raster.
/// let stats = hs_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
/// println!("{stats:#?}");
/// # Ok(())
/// # }
/// ```
/// The resulting output is:
///
/// ```text
/// StatisticsAll {
///     min: 50.0,
///     max: 255.0,
///     mean: 206.85964128690114,
///     std_dev: 52.73836661993173,
/// }
/// ```
/// See: [`gdaldem color-relief`](https://gdal.org/programs/gdaldem.html#color-relief) for details,
///
pub fn color_relief<P: AsRef<Path>>(
    ds: &Dataset,
    dest_file: P,
    options: &ColorReliefOptions,
) -> Result<Dataset> {
    let colors = options.color_config();
    dem_eval(
        ds,
        dest_file.as_ref(),
        DemAlg::ColorRelief,
        &options.to_options_list()?,
        Some(colors),
    )
}

/// Performs hill-shade rendering of DEM data.
///
/// This routine outputs an 8-bit raster with a nice shaded relief effect.
/// It’s very useful for visualizing the terrain.
/// You can optionally specify the azimuth and altitude of the light source,
/// a vertical exaggeration factor and a scaling factor to account for
/// differences between vertical and horizontal units.
///
/// The value `0` is used as the output no-data value.
///
/// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
///
/// # Example
///
/// ```rust, no_run
/// use gdal::Dataset;
/// use gdal::raster::dem::*;
/// # fn main() -> gdal::errors::Result<()> {
/// use std::path::Path;
/// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
/// let mut opts = HillshadeOptions::new();
/// opts
///     .with_algorithm(DemSlopeAlg::Horn)
///     .with_z_factor(4.0)
///     .with_scale(98473.0)
///     .with_shading_mode(ShadingMode::Combined);
/// let hs_ds = hillshade(&ds, Path::new("target/dem-hills-shade.tiff"), &opts)?;
/// let stats = hs_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
/// println!("{stats:#?}");
/// # Ok(())
/// # }
/// ```
/// The resulting output is:
///
/// ```text
/// StatisticsAll {
///     min: 31.0,
///     max: 255.0,
///     mean: 234.71988886841396,
///     std_dev: 30.556285572761446,
/// }
/// ```
/// See: [`gdaldem hillshade`](https://gdal.org/programs/gdaldem.html#hillshade) for details,
///
pub fn hillshade<P: AsRef<Path>>(
    ds: &Dataset,
    dest_file: P,
    options: &HillshadeOptions,
) -> Result<Dataset> {
    dem_eval(
        ds,
        dest_file.as_ref(),
        DemAlg::Hillshade,
        &options.to_options_list()?,
        None,
    )
}

/// Roughness routine for DEM datasets.
///
/// This processor outputs a single-band raster with values computed from the elevation.
/// Roughness is the largest inter-cell difference of a central pixel and its surrounding cell,
/// as defined in Wilson et al (2007, Marine Geodesy 30:3-35).
///
/// The value `-9999` is used as the output no-data value.
///
/// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
///
/// # Example
///
/// ```rust, no_run
/// # fn main() -> gdal::errors::Result<()> {
/// use gdal::Dataset;
/// use std::path::Path;
/// use gdal::raster::dem::*;
/// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
/// let roughness_ds = roughness(&ds, Path::new("target/dem-hills-roughness.tiff"), &RoughnessOptions::default())?;
/// let stats = roughness_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
/// println!("{stats:#?}");
/// # Ok(())
/// # }
/// ```
/// The resulting output is:
///
/// ```text
/// StatisticsAll {
///     min: 0.0,
///     max: 14.361007690429688,
///     mean: 1.5128357817365072,
///     std_dev: 2.0120679959607686,
/// }
/// ```
///
/// See: [`gdaldem roughness`](https://gdal.org/programs/gdaldem.html#roughness) for details.
pub fn roughness<P: AsRef<Path>>(
    ds: &Dataset,
    dest_file: P,
    options: &RoughnessOptions,
) -> Result<Dataset> {
    dem_eval(
        ds,
        dest_file.as_ref(),
        DemAlg::Roughness,
        &options.to_options_list()?,
        None,
    )
}

/// Slope computation routine for DEM datasets.
///
/// This method will take a DEM raster Dataset and output a 32-bit float raster with slope values.
///
/// You have the option of specifying the type of slope value you want:
/// [degrees or percent slope](SlopeOptions::with_percentage_results).
///
/// In cases where the horizontal units differ from the vertical units, you can also supply
/// a [scaling factor](SlopeOptions::with_scale).
///
/// The value `-9999` is used as the output no-data value.
///
/// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
///
/// # Example
///
/// ```rust, no_run
/// # fn main() -> gdal::errors::Result<()> {
/// use std::path::Path;
/// use gdal::Dataset;
/// use gdal::raster::dem::*;
/// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
/// let mut opts = SlopeOptions::new();
/// opts
///     .with_algorithm(DemSlopeAlg::Horn)
///     .with_percentage_results(true)
///     .with_scale(98473.0);
/// let slope_ds = slope(&ds, Path::new("target/dem-hills-slope.tiff"), &opts)?;
/// let stats = slope_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
/// println!("{stats:#?}");
/// # Ok(())
/// # }
/// ```
/// The resulting output is:
///
/// ```text
/// StatisticsAll {
///     min: 0.0,
///     max: 65.44061279296875,
///     mean: 6.171115265248098,
///     std_dev: 8.735612161193623,
/// }
/// ```
///
/// See: [`gdaldem slope`](https://gdal.org/programs/gdaldem.html#slope) for details,
pub fn slope<P: AsRef<Path>>(
    ds: &Dataset,
    dest_file: P,
    options: &SlopeOptions,
) -> Result<Dataset> {
    dem_eval(
        ds,
        dest_file.as_ref(),
        DemAlg::Slope,
        &options.to_options_list()?,
        None,
    )
}

/// Topographic Position Index (TPI) routine for DEM datasets
///
/// This method outputs a single-band raster with values computed from the elevation.
/// A Topographic Position Index is defined as the difference between a central pixel and the
/// mean of its surrounding cells (see Wilson et al 2007, Marine Geodesy 30:3-35).
///
/// The value `-9999` is used as the output no-data value.
///
/// Note: Results are nonsensical if the underlying [`Dataset`] does not contain digital elevation data.
///
/// # Example
///
/// ```rust, no_run
/// # fn main() -> gdal::errors::Result<()> {
/// use std::path::Path;
/// use gdal::Dataset;
/// use gdal::raster::dem::*;
/// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
/// let tpi_ds = topographic_position_index(&ds, Path::new("target/dem-hills-tpi.tiff"), &TpiOptions::default())?;
/// let stats = tpi_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
/// println!("{stats:#?}");
/// # Ok(())
/// # }
/// ```
/// The resulting output is:
///
/// ```text
/// StatisticsAll {
///     min: -4.7376708984375,
///     max: 4.7724151611328125,
///     mean: 0.00012131847966825689,
///     std_dev: 0.48943078832474257,
/// }
/// ```
///
/// See: [`gdaldem tpi`](https://gdal.org/programs/gdaldem.html#tpi) for details.
pub fn topographic_position_index<P: AsRef<Path>>(
    ds: &Dataset,
    dest_file: P,
    options: &TpiOptions,
) -> Result<Dataset> {
    dem_eval(
        ds,
        dest_file.as_ref(),
        DemAlg::Tpi,
        &options.to_options_list()?,
        None,
    )
}

/// Terrain Ruggedness Index (TRI) routine for DEM datasets
///
/// This method outputs a single-band raster with values computed from the elevation.
/// TRI stands for Terrain Ruggedness Index, which measures the difference between a
/// central pixel and its surrounding cells.
///
/// The value `-9999` is used as the output no-data value.
///
/// # Example
///
/// ```rust, no_run
/// # fn main() -> gdal::errors::Result<()> {
/// use std::path::Path;
/// use gdal::Dataset;
/// use gdal::raster::dem::*;
/// let ds = Dataset::open("fixtures/dem-hills.tiff")?;
/// let mut opts = TriOptions::new();
/// opts.with_algorithm(DemTriAlg::Wilson);
/// let tri_ds = terrain_ruggedness_index(&ds, Path::new("target/dem-hills-tri.tiff"), &opts)?;
/// let stats = tri_ds.rasterband(1)?.get_statistics(true, false)?.unwrap();
/// println!("{stats:#?}");
/// # Ok(())
/// # }
/// ```
/// The resulting output is:
///
/// ```text
/// StatisticsAll {
///     min: 0.0,
///     max: 4.983623504638672,
///     mean: 0.49063101456532326,
///     std_dev: 0.6719356336694824,
/// }
/// ```
///
/// See: [`gdaldem tri`](https://gdal.org/programs/gdaldem.html#tri) for details.
pub fn terrain_ruggedness_index<P: AsRef<Path>>(
    ds: &Dataset,
    dest_file: P,
    options: &TriOptions,
) -> Result<Dataset> {
    dem_eval(
        ds,
        dest_file.as_ref(),
        DemAlg::Tri,
        &options.to_options_list()?,
        None,
    )
}

/// Execute the processor on the given [`Dataset`].
fn dem_eval(
    src: &Dataset,
    dst_file: &Path,
    alg: DemAlg,
    options: &CslStringList,
    color_relief_config: Option<&Path>,
) -> Result<Dataset> {
    let popts = options::GdalDEMProcessingOptions::new(options)?;
    let mode = CString::new(alg.to_gdal_option())?;
    let dest = _path_to_c_string(dst_file)?;
    let cfile = color_relief_config.and_then(|p| _path_to_c_string(p).ok());
    let cfile_ptr = cfile.as_deref().map(CStr::as_ptr).unwrap_or(ptr::null());

    let mut pb_usage_error: c_int = 0;
    let out_ds = unsafe {
        // Docs: https://github.com/OSGeo/gdal/blob/6a3584b2fea51f92022d24ad8036749ba1b98958/apps/gdaldem_lib.cpp#L3184
        GDALDEMProcessing(
            dest.as_ptr(),
            src.c_dataset(),
            mode.as_ptr(),
            cfile_ptr,
            popts.as_ptr(),
            &mut pb_usage_error as *mut c_int,
        )
    };

    if pb_usage_error != 0 {
        Err(_last_cpl_err(CPLErr::CE_Failure))
    } else {
        let out_ds = unsafe { Dataset::from_c_dataset(out_ds) };
        Ok(out_ds)
    }
}
