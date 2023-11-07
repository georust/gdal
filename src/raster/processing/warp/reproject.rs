use crate::errors::Result;
use crate::raster::processing::warp::GdalWarpOptions;
use crate::spatial_ref::SpatialRef;
use crate::utils::{_last_cpl_err, _path_to_c_string};
use crate::{Dataset, DriverManager};
use gdal_sys::CPLErr;
use std::ffi::CString;
use std::path::Path;
use std::ptr;

/// Optional settings for GDAL Warp-based reprojection.
#[derive(Debug, Clone, Default)]
pub struct ReprojectOptions {
    warp_options: GdalWarpOptions,
    max_error: Option<f64>,
    src_srs: Option<SpatialRef>,
    src_nodata: Option<f64>,
    dst_nodata: Option<f64>,
    output_format: Option<String>,
}

impl ReprojectOptions {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the maximum error.
    ///
    /// Measured in input pixels, it is the allowed in approximating
    /// transformations.
    ///
    /// `0.0` indicates exact calculations.
    pub fn with_max_error(&mut self, max_error: f64) -> &mut Self {
        self.max_error = Some(max_error);
        self
    }

    /// Fetch the specified maximum error.
    ///
    /// Returns `None` if unset.
    pub fn max_error(&self) -> Option<f64> {
        self.max_error
    }

    /// Specify the source projection.
    ///
    /// If unset, the source projection is read from the source dataset.
    /// If set, the source projection os overridden.
    pub fn with_src_projection(&mut self, srs: &SpatialRef) -> &mut Self {
        self.src_srs = Some(srs.clone());
        self
    }

    /// Fetch the specified source projection, if any.
    pub fn src_projection(&self) -> Option<&SpatialRef> {
        self.src_srs.as_ref()
    }

    /// Specify the source no-data value.
    ///
    /// Overrides any no-data value specified in the source dataset.
    pub fn with_src_nodata(&mut self, nodata_value: f64) -> &mut Self {
        self.src_nodata = Some(nodata_value);
        self
    }

    /// Specify the destination no-data value.
    pub fn with_dst_nodata(&mut self, nodata_value: f64) -> &mut Self {
        self.dst_nodata = Some(nodata_value);
        self
    }

    /// Explicitly specify output raster format.
    ///
    /// This is equivalent to the `-of <format>` CLI flag accepted by many GDAL tools.
    ///
    /// The value of `format` must be the identifier of a driver supported by the runtime
    /// environment's GDAL library (e.g. `COG`, `JPEG`, `VRT`, etc.). A list of these identifiers
    /// is available from `gdalinfo --formats`:
    ///
    /// ```text
    /// ❯ gdalinfo --formats
    /// Supported Formats:
    ///   VRT -raster,multidimensional raster- (rw+v): Virtual Raster
    ///   DERIVED -raster- (ro): Derived datasets using VRT pixel functions
    ///   GTiff -raster- (rw+vs): GeoTIFF
    ///   COG -raster- (wv): Cloud optimized GeoTIFF generator
    ///   NITF -raster- (rw+vs): National Imagery Transmission Format
    /// ...
    /// ```
    /// If no output format is specified, then the driver from the source dataset is used.
    ///
    pub fn with_output_format(&mut self, format: &str) -> &mut Self {
        self.output_format = Some(format.to_owned());
        self
    }

    /// Fetch the specified output format driver identifier, if any.
    pub fn output_format(&self) -> Option<String> {
        self.output_format.clone()
    }

    /// Fetch an immutable reference to the general Warp options.
    pub fn warp_options(&self) -> &GdalWarpOptions {
        &self.warp_options
    }

    /// Fetch a mutable reference to the general Warp options.
    pub fn warp_options_mut(&mut self) -> &mut GdalWarpOptions {
        &mut self.warp_options
    }
}

pub(super) fn reproject(
    src: &Dataset,
    dst_file: &Path,
    dst_projection: &SpatialRef,
    options: &ReprojectOptions,
) -> Result<()> {
    let dest = _path_to_c_string(dst_file)?;
    // Format the destination projection.
    let dst_wkt = CString::new(dst_projection.to_wkt()?)?;
    // Format the source projection, if specified.
    let src_wkt = options
        .src_projection()
        .map(|s| s.to_wkt())
        .transpose()?
        .map(CString::new)
        .transpose()?;
    let src_wkt_ptr = src_wkt.map(|s| s.as_ptr()).unwrap_or(ptr::null());

    let max_error = options.max_error().unwrap_or(0.0);

    let driver = options
        .output_format
        .as_ref()
        .map(|f| DriverManager::get_driver_by_name(f))
        .transpose()?
        .unwrap_or(src.driver());

    // GDALCreateAndReprojectImage requires a mutable pointer to
    // an GDALWarpOptions instance. We could either propagate mutability up the call chain
    // or clone the given options. Given the user may want to reuse settings for consistent
    // application across multiple files and may find mutation unexpected, we clone make a clone.
    let mut warp_options = options.warp_options().clone();

    // If no-data values are specified, we need to initialize some state in
    // `GdalWarpOptions` first.
    if options.src_nodata.is_some() || options.dst_nodata.is_some() {
        warp_options.with_band_count(src.raster_count() as usize);
    }

    if let Some(src_nodata) = options.src_nodata {
        warp_options.apply_src_nodata(src_nodata)?;
    }

    if let Some(dst_nodata) = options.dst_nodata {
        warp_options.apply_dst_nodata(dst_nodata)?;
    }

    if warp_options.working_datatype().is_none() {
        warp_options.with_auto_working_datatype();
    }

    println!("{warp_options}");
    let rv = unsafe {
        // See: https://github.com/OSGeo/gdal/blob/7b6c3fe71d61699abe66ea372bcd110701e38ff3/alg/gdalwarper.cpp#L235
        gdal_sys::GDALCreateAndReprojectImage(
            src.c_dataset(),
            src_wkt_ptr,
            dest.as_ptr(),
            dst_wkt.as_ptr(),
            driver.c_driver(),
            ptr::null_mut(),
            warp_options.resampling_alg().to_gdal(),
            warp_options.memory_limit() as f64,
            max_error,
            None,
            ptr::null_mut(),
            warp_options.as_ptr_mut(),
        )
    };

    if rv != CPLErr::CE_None {
        return Err(_last_cpl_err(rv));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::errors::Result;
    use crate::raster::processing::warp::reproject::ReprojectOptions;
    use crate::raster::processing::warp::resample::WarpResampleAlg;
    use crate::raster::processing::warp::WarpProcessing;
    use crate::spatial_ref::SpatialRef;
    use crate::test_utils::{fixture, TempFixture};
    use crate::{assert_near, Dataset};
    use std::path::Path;

    #[test]
    fn reproject() -> Result<()> {
        // Expected raster created with:
        //     gdalwarp -overwrite -t_srs EPSG:4269 -dstnodata 255 -r near -of GTiff fixtures/labels.tif fixtures/labels-nad.tif
        let expected = Dataset::open(fixture("labels-nad.tif"))?;
        let dst_srs = SpatialRef::from_epsg(4269)?;
        let source = TempFixture::fixture("labels.tif");
        //let dest = source.path().parent().unwrap().join("labels-proj.tif");
        let dest = Path::new("target").join("labels-proj.tif");
        let ds = Dataset::open(&source)?;
        let mut opts = ReprojectOptions::default();
        opts.with_output_format("GTiff")
            .with_dst_nodata(255.0)
            .warp_options_mut()
            .with_resampling_alg(WarpResampleAlg::NearestNeighbour);

        ds.reproject(&dest, &dst_srs, &opts)?;

        let result = Dataset::open(dest)?;
        let result_stats = result.rasterband(1)?.get_statistics(true, false)?.unwrap();
        dbg!(&result_stats);

        let expected_stats = expected
            .rasterband(1)?
            .get_statistics(true, false)?
            .unwrap();
        dbg!(&expected_stats);

        assert_near!(StatisticsAll, result_stats, expected_stats, epsilon = 1e-4);

        Ok(())
    }
}
