//! GDAL Warp API Bindings
//!
//! See also:
//! * [Warper C++ API](https://gdal.org/api/gdalwarp_cpp.html)
//! * [Warp API Tutorial](https://gdal.org/tutorials/warp_tut.html)
//! * [`gdalwarp` Program](https://gdal.org/programs/gdalwarp.html#gdalwarp)

mod reproject_options;
mod resample;
mod warp_options;

use gdal_sys::{CPLErr, GDALDatasetH, GDALWarp};
pub use reproject_options::*;
pub use resample::*;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::ptr;
pub use warp_options::*;

use crate::dataset::Dataset;
use crate::DriverManager;

use crate::errors::*;
use crate::spatial_ref::SpatialRef;
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _path_to_c_string};

/// Reproject raster dataset into the given [`SpatialRef`] and save result to `dst_file`.
pub fn create_and_reproject<P: AsRef<Path>>(
    ds: &Dataset,
    dst_file: P,
    dst_srs: &SpatialRef,
    options: &ReprojectOptions,
) -> Result<()> {
    let dest_file = dst_file.as_ref();
    fn reproject(
        src: &Dataset,
        dst_file: &Path,
        dst_srs: &SpatialRef,
        options: &ReprojectOptions,
    ) -> Result<()> {
        let dest = _path_to_c_string(dst_file)?;
        // Format the destination projection.
        let dst_wkt = CString::new(dst_srs.to_wkt()?)?;
        // Format the source projection, if specified.
        let src_wkt = options
            .src_spatial_ref()
            .map(|s| s.to_wkt())
            .transpose()?
            .map(CString::new)
            .transpose()?;
        let src_wkt_ptr = src_wkt.map(|s| s.as_ptr()).unwrap_or(ptr::null());

        let driver = options
            .output_format()
            .as_ref()
            .map(|f| DriverManager::get_driver_by_name(f))
            .transpose()?
            .unwrap_or(src.driver());

        let mut warp_options = options.clone_and_init_warp_options(src.raster_count())?;

        let rv = unsafe {
            // See: https://github.com/OSGeo/gdal/blob/7b6c3fe71d61699abe66ea372bcd110701e38ff3/alg/gdalwarper.cpp#L235
            gdal_sys::GDALCreateAndReprojectImage(
                src.c_dataset(),
                src_wkt_ptr,
                dest.as_ptr(),
                dst_wkt.as_ptr(),
                driver.c_driver(),
                ptr::null_mut(), // create options
                warp_options.resampling_alg().to_gdal(),
                warp_options.memory_limit() as f64,
                options.max_error().unwrap_or(0.0),
                None,            // progress fn
                ptr::null_mut(), // progress arg
                warp_options.as_ptr_mut(),
            )
        };

        if rv != CPLErr::CE_None {
            return Err(_last_cpl_err(rv));
        }

        // See https://lists.osgeo.org/pipermail/gdal-dev/2023-November/057887.html for
        // why this is required. To get around it We should rewrite this function to use the
        // lower-level `GDALWarp` API.
        if options.dst_nodata().is_some() {
            let ds = Dataset::open(dst_file)?;
            for b in 1..=ds.raster_count() {
                let mut rb = ds.rasterband(b)?;
                rb.set_no_data_value(options.dst_nodata())?;
            }
        }

        Ok(())
    }
    reproject(ds, dest_file, dst_srs, options)
}

/// Reproject one dataset into another dataset.
///
/// Assumes destination dataset is properly sized and setup with a [`SpatialRef`],
/// [`GeoTransform`][crate::GeoTransform], [`RasterBand`][crate::raster::RasterBand], etc.
///
/// See [`create_and_reproject`] for a more flexible alternative.
pub fn reproject_into(
    src: &Dataset,
    dst: &mut Dataset,
    options: &ReprojectIntoOptions,
) -> Result<()> {
    // Format the source projection, if specified.
    let src_wkt = options
        .src_spatial_ref()
        .map(|s| s.to_wkt())
        .transpose()?
        .map(CString::new)
        .transpose()?;
    let src_wkt_ptr = src_wkt.map(|s| s.as_ptr()).unwrap_or(ptr::null());

    // Format the destination projection, if specified.
    let dst_wkt = options
        .src_spatial_ref()
        .map(|s| s.to_wkt())
        .transpose()?
        .map(CString::new)
        .transpose()?;
    let dst_wkt_ptr = dst_wkt.map(|s| s.as_ptr()).unwrap_or(ptr::null());

    // GDALCreateAndReprojectImage requires a mutable pointer to
    // an GDALWarpOptions instance. We could either propagate mutability up the call chain
    // or clone the given options. Given the user may want to reuse settings for consistent
    // application across multiple files and may find mutation unexpected, we clone make a clone.
    let mut warp_options = options.clone_and_init_warp_options(dst.raster_count())?;

    let rv = unsafe {
        gdal_sys::GDALReprojectImage(
            src.c_dataset(),
            src_wkt_ptr,
            dst.c_dataset(),
            dst_wkt_ptr,
            warp_options.resampling_alg().to_gdal(),
            warp_options.memory_limit() as f64,
            options.max_error().unwrap_or(0.0),
            None,            // progress fn
            ptr::null_mut(), // progress arg
            warp_options.as_ptr_mut(),
        )
    };
    if rv != CPLErr::CE_None {
        return Err(_last_cpl_err(rv));
    }

    // See https://lists.osgeo.org/pipermail/gdal-dev/2023-November/057887.html for
    // why this is required. To get around it We should rewrite this function to use the
    // lower-level `GDALWarp` API.
    if options.dst_nodata().is_some() {
        for b in 1..=dst.raster_count() {
            let mut rb = dst.rasterband(b)?;
            rb.set_no_data_value(options.dst_nodata())?;
        }
    }

    Ok(())
}

pub fn warp<D>(source: &Dataset, dest: D, options: &GdalWarpOptions) -> Result<Dataset>
where
    D: Into<WarpDestination>,
{
    warp_multiple(&[source], dest, options)
}

pub fn warp_multiple<D>(source: &[&Dataset], dest: D, options: &GdalWarpOptions) -> Result<Dataset>
where
    D: Into<WarpDestination>,
{
    let app_opts = GdalWarpAppOptions::default();

    if true {
        todo!("how the hell do you go from {options:?} to GdalWarpAppOptions?");
    }

    let mut source = source.iter().map(|ds| ds.c_dataset()).collect::<Vec<_>>();

    let dest = dest.into();
    match dest {
        WarpDestination::Dataset(ds) => {
            let ds_c = unsafe {
                GDALWarp(
                    ptr::null_mut(),
                    ds.c_dataset(),
                    source.len() as libc::c_int,
                    source.as_mut_ptr(),
                    app_opts.as_ptr(),
                    ptr::null_mut(),
                )
            };
            if ds_c.is_null() {
                Err(_last_null_pointer_err("GDALWarp"))
            } else {
                Ok(ds)
            }
        }
        WarpDestination::Path(p) => {
            let path = _path_to_c_string(&p)?;
            let ds_c = unsafe {
                GDALWarp(
                    path.as_ptr(),
                    ptr::null_mut(),
                    source.len() as libc::c_int,
                    source.as_ptr() as *mut GDALDatasetH,
                    app_opts.as_ptr(),
                    ptr::null_mut(),
                )
            };
            Ok(unsafe { Dataset::from_c_dataset(ds_c) })
        }
    }
}

#[derive(Debug)]
pub enum WarpDestination {
    Dataset(Dataset),
    Path(PathBuf),
}

impl From<Dataset> for WarpDestination {
    fn from(ds: Dataset) -> Self {
        WarpDestination::Dataset(ds)
    }
}

impl From<PathBuf> for WarpDestination {
    fn from(path: PathBuf) -> Self {
        WarpDestination::Path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Result;
    use crate::raster::GdalDataType;
    use crate::spatial_ref::SpatialRef;
    use crate::test_utils::{fixture, InMemoryFixture, TempFixture};
    use crate::{assert_near, Dataset};

    // TODO: For some unknown reason this test fails on GDAL < 3.4
    #[cfg(any(all(major_ge_3, minor_ge_4), major_ge_4))]
    #[test]
    fn test_create_reproject() -> Result<()> {
        use std::path::Path;
        let dst_srs = SpatialRef::from_epsg(4269)?;
        let source = TempFixture::fixture("labels.tif");

        let dest = Path::new("target").join("labels-proj.tif");
        let ds = Dataset::open(&source)?;

        let mut opts = ReprojectOptions::default();
        opts.with_output_format("GTiff")
            .with_dst_nodata(255.0)
            .warp_options_mut()
            .with_initial_value(InitValue::NoData)
            .with_resampling_alg(WarpResampleAlg::NearestNeighbour);

        create_and_reproject(&ds, &dest, &dst_srs, &opts)?;

        let result = Dataset::open(dest)?;
        let rb = result.rasterband(1)?;
        let result_stats = rb.get_statistics(true, false)?.unwrap();

        // Expected raster created with:
        //     gdalwarp -overwrite -t_srs EPSG:4269 -dstnodata 255 -r near -of GTiff fixtures/labels.tif fixtures/labels-nad.tif
        let expected = Dataset::open(fixture("labels-nad.tif"))?;
        let erb = expected.rasterband(1)?;
        assert_eq!(erb.no_data_value(), Some(255.0));
        assert_eq!(erb.band_type(), GdalDataType::UInt8);

        let expected_stats = erb.get_statistics(true, false)?.unwrap();
        assert_near!(StatisticsAll, result_stats, expected_stats, epsilon = 1e-2);

        Ok(())
    }

    #[test]
    fn test_reproject_into() -> Result<()> {
        let source = TempFixture::fixture("labels.tif");
        let source_ds = Dataset::open(&source)?;

        let drv = DriverManager::get_driver_by_name("GTiff")?;
        let outfile = InMemoryFixture::new("foo.tif");
        let dst_srs = SpatialRef::from_epsg(4269)?;
        let mut dest_ds = drv.create_with_band_type::<u8, _>(outfile.path(), 210, 151, 1)?;
        dest_ds.set_spatial_ref(&dst_srs)?;
        dest_ds.set_geo_transform(&[
            -78.66496151541256,
            0.0003095182591293914,
            0.0,
            38.41639646432918,
            0.0,
            -0.0003095182591293914,
        ])?;

        let mut opts = ReprojectIntoOptions::default();
        opts.with_dst_nodata(255.0)
            .warp_options_mut()
            .with_initial_value(InitValue::NoData)
            .with_resampling_alg(WarpResampleAlg::NearestNeighbour);

        reproject_into(&source_ds, &mut dest_ds, &opts)?;

        let rb = dest_ds.rasterband(1)?;
        let result_stats = rb.get_statistics(true, false)?.unwrap();

        // Expected raster created with:
        //     gdalwarp -overwrite -t_srs EPSG:4269 -dstnodata 255 -r near -of GTiff fixtures/labels.tif fixtures/labels-nad.tif
        let expected = Dataset::open(fixture("labels-nad.tif"))?;
        let erb = expected.rasterband(1)?;
        assert_eq!(erb.no_data_value(), Some(255.0));
        assert_eq!(erb.band_type(), GdalDataType::UInt8);

        let expected_stats = erb.get_statistics(true, false)?.unwrap();
        assert_near!(StatisticsAll, result_stats, expected_stats, epsilon = 1e-2);

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_warp() -> Result<()> {
        let source = TempFixture::fixture("labels.tif");
        let source_ds = Dataset::open(&source)?;
        let dest = Path::new("target").join("labels-warp.tif");

        let mut options = GdalWarpOptions::default();
        options
            .with_band_count(source_ds.raster_count())
            .with_initial_value(InitValue::NoData)
            .with_resampling_alg(WarpResampleAlg::NearestNeighbour);

        warp(&source_ds, dest, &options)?;
        Ok(())
    }
}
