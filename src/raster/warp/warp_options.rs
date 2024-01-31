use std::fmt::{Debug, Display, Formatter};
use std::ptr;
use std::ptr::NonNull;

use crate::cpl::CslStringList;
use crate::errors::{GdalError, Result};
use crate::raster::warp::resample::WarpResampleAlg;
use crate::raster::GdalDataType;
use crate::utils::_last_null_pointer_err;
use crate::xml::GdalXmlNode;
use gdal_sys::{
    GDALCloneWarpOptions, GDALCreateWarpOptions, GDALDeserializeWarpOptions,
    GDALDestroyWarpOptions, GDALSerializeWarpOptions, GDALWarpAppOptions, GDALWarpAppOptionsFree,
    GDALWarpAppOptionsNew, GDALWarpInitDefaultBandMapping, GDALWarpInitDstNoDataReal,
    GDALWarpInitSrcNoDataReal, GDALWarpOptions, GDALWarpResolveWorkingDataType,
};
use libc::c_char;

/// Container for options provided to GDAL Warp routines.
///
/// See: [`GDALWarpOptions`](https://gdal.org/api/gdalwarp_cpp.html#_CPPv415GDALWarpOptions)
/// for details.
pub struct GdalWarpOptions(NonNull<GDALWarpOptions>);

impl GdalWarpOptions {
    pub fn new() -> Self {
        unsafe { Self::from_ptr(GDALCreateWarpOptions()) }
    }

    /// Create Self from a raw pointer.
    ///
    /// # Safety
    /// Caller is responsible for ensuring `ptr` is not null, and
    /// ownership of `ptr` is properly transferred
    pub unsafe fn from_ptr(ptr: *mut GDALWarpOptions) -> Self {
        Self(NonNull::new_unchecked(ptr))
    }

    /// Specify the resampling algorithm to use in Warp operation.
    pub fn with_resampling_alg(&mut self, alg: WarpResampleAlg) -> &mut Self {
        unsafe { (*self.as_ptr_mut()).eResampleAlg = alg.to_gdal() };
        self
    }

    /// Get the resampling algorithm to be used in Warp operation.
    pub fn resampling_alg(&self) -> WarpResampleAlg {
        // `unwrap` below is ok because `with_resampling_alg` is the only way it got set,
        // aside from the GDAL default, which is `GRA_NearestNeighbour`.
        WarpResampleAlg::from_gdal(unsafe { (*self.as_ptr()).eResampleAlg }).unwrap_or_default()
    }

    /// Set the datatype used during processing.
    ///
    /// If unset, the algorithm picks the datatype.
    pub fn with_working_datatype(&mut self, dt: GdalDataType) -> &mut Self {
        unsafe { (*self.as_ptr_mut()).eWorkingDataType = dt.gdal_ordinal() };
        self
    }

    /// Fetch working datatype, if specified.
    pub fn working_datatype(&self) -> Option<GdalDataType> {
        let c_dt = unsafe { (*self.as_ptr()).eWorkingDataType };
        let dt: GdalDataType = c_dt.try_into().ok()?;

        // Default is `Unknown`, so we consider that "unspecified".
        if dt == GdalDataType::Unknown {
            None
        } else {
            Some(dt)
        }
    }

    /// This option is a convenience method for setting `INIT_DEST` setting via [`extra_options_mut`].
    ///
    /// It forces the destination image to be initialized to the indicated value (for all bands),
    /// or indicates that it should be initialized to the band's no-data value.
    ///
    /// If this value isn't set the destination image will be read and overlaid.
    ///
    /// See: [`GDALWarpOptions::papszWarpOptions](https://gdal.org/api/gdalwarp_cpp.html#_CPPv4N15GDALWarpOptions16papszWarpOptionsE)
    pub fn with_initial_value(&mut self, init: InitValue) -> &mut Self {
        self.extra_options_mut()
            .set_name_value("INIT_DEST", &init.to_string())
            .expect("Failed to set INIT_DEST"); // We can unwrap because we know the strings are valid.
        self
    }

    /// Fetch the initial value setting, if any.
    ///
    /// See [`with_initial_value`][Self::with_initial_value] for details.
    pub fn initial_value(&self) -> Option<InitValue> {
        let init = self.extra_options().fetch_name_value("INIT_DEST");

        match init.as_deref() {
            Some("NO_DATA") => Some(InitValue::NoData),
            Some(s) => s.parse::<f64>().ok().map(InitValue::Value),
            None => None,
        }
    }

    /// If the working data type is unknown, this method will determine a valid working
    /// data type to support the data in the src and dest data sets and any noData values.
    pub fn with_auto_working_datatype(&mut self) -> &mut Self {
        unsafe { GDALWarpResolveWorkingDataType(self.as_ptr_mut()) };
        self
    }

    /// Memory limit in in bytes,
    ///
    /// Use `0` to specify GDAL default.
    pub fn with_memory_limit(&mut self, limit_bytes: usize) -> &mut Self {
        unsafe { (*self.as_ptr_mut()).dfWarpMemoryLimit = limit_bytes as f64 };
        self
    }

    /// Fetch the memory limit setting in bytes.
    ///
    /// Zero means use GDAL default.
    pub fn memory_limit(&self) -> usize {
        unsafe { (*self.as_ptr()).dfWarpMemoryLimit as usize }
    }

    /// Number of bands to process
    ///
    /// `0` selects all bands.
    pub fn with_band_count(&mut self, num_bands: usize) -> &mut Self {
        unsafe { GDALWarpInitDefaultBandMapping(self.as_ptr_mut(), num_bands as libc::c_int) };
        self
    }

    /// Get the specified number of bands to process
    ///
    /// `0` indicates all bands.
    pub fn band_count(&mut self) -> usize {
        let cnt = unsafe { (*self.as_ptr()).nBandCount };
        cnt as usize
    }

    /// Sets the source Dataset no-data value. Internal use only.
    ///
    /// This method exists to facilitate delaying the specification of a homogeneous no-data value
    /// until the number of bands is known (at the point of warp call), via `with_band_count`,
    /// which initializes the band mapping, as required by GDALWarp.
    ///
    /// Therefore, the caller of this method is responsible for ensuring that a non-zero number of bands has been
    /// first set via [`with_band_count`][Self::with_band_count]. If not, this method will return
    /// `Err(GdalError::UnexpectedLogicError(...))`
    pub(super) fn apply_src_nodata(&mut self, no_data_value: f64) -> Result<&mut Self> {
        if self.band_count() == 0 {
            return Err(GdalError::UnexpectedLogicError(
                "Specification of source no-data value prior to initializing band mapping via `with_band_count`".into())
            );
        }

        // GDALWarpOptions destructor frees this. See:
        // https://github.com/OSGeo/gdal/blob/a9635785a2db8f575328326f2b1833e743ec8828/alg/gdalwarper.cpp#L1293
        unsafe { GDALWarpInitSrcNoDataReal(self.as_ptr_mut(), no_data_value) };

        Ok(self)
    }

    /// Sets the destination Dataset no-data value. Internal use only.
    ///
    /// See [`Self::apply_src_nodata`] for additional details.
    pub(super) fn apply_dst_nodata(&mut self, no_data_value: f64) -> Result<&mut Self> {
        if self.band_count() == 0 {
            return Err(GdalError::UnexpectedLogicError(
                "Specification of destination no-data value prior to initializing band mapping via `with_band_count`".into())
            );
        }

        // The GDALWarpOptions destructor frees this. See:
        // https://github.com/OSGeo/gdal/blob/a9635785a2db8f575328326f2b1833e743ec8828/alg/gdalwarper.cpp#L1295
        unsafe { GDALWarpInitDstNoDataReal(self.as_ptr_mut(), no_data_value) };

        Ok(self)
    }

    /// Get any extra options attached to the Warp options.
    pub fn extra_options(&self) -> &CslStringList {
        let opts_array: &*mut *mut c_char = unsafe { &(*self.as_ptr()).papszWarpOptions };
        // Proof that GDALWarpOptions owns the CslStringList, and we just need to wrap it:
        // https://github.com/OSGeo/gdal/blob/a9635785a2db8f575328326f2b1833e743ec8828/alg/gdalwarper.cpp#L1290
        // `CslStringList` is `transparent` with a single field, so this should be ok.
        unsafe { std::mem::transmute(opts_array) }
    }

    /// Get a mutable reference to extra options attached to the Warp options.
    pub fn extra_options_mut(&mut self) -> &mut CslStringList {
        let opts_array: &*mut *mut c_char = unsafe { &(*self.as_ptr()).papszWarpOptions };
        // See `extra_options` for rationale on transmute.
        let csl: *mut CslStringList = opts_array as *const *mut *mut i8 as *mut CslStringList;
        // `unwrap` should be ok because `opts_array` points to an offset against `self`, and
        // we can assume `self` is not null.
        unsafe { csl.as_mut().unwrap() }
    }

    /// Serialize settings to GDAL XML.
    pub fn to_xml(&self) -> Result<GdalXmlNode> {
        let c_xml = unsafe { GDALSerializeWarpOptions(self.as_ptr()) };
        Ok(unsafe { GdalXmlNode::from_ptr(c_xml) })
    }

    /// Deserialize options from GDAL XML
    pub fn from_xml(xml: &GdalXmlNode) -> Result<Self> {
        let c_opts = unsafe { GDALDeserializeWarpOptions(xml.as_ptr_mut()) };
        if c_opts.is_null() {
            Err(_last_null_pointer_err("GDALDeserializeWarpOptions"))
        } else {
            Ok(unsafe { Self::from_ptr(c_opts) })
        }
    }

    /// Get a immutable pointer to C API options.
    pub fn as_ptr(&self) -> *const GDALWarpOptions {
        self.0.as_ptr()
    }

    /// Get a mutable pointer to C API options.
    pub fn as_ptr_mut(&mut self) -> *mut GDALWarpOptions {
        self.0.as_ptr()
    }
}

impl Clone for GdalWarpOptions {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr(GDALCloneWarpOptions(self.as_ptr())) }
    }
}

impl Drop for GdalWarpOptions {
    fn drop(&mut self) {
        unsafe { GDALDestroyWarpOptions(self.as_ptr_mut()) }
    }
}

impl Default for GdalWarpOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for GdalWarpOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let xml = self.to_xml().map_err(|_| std::fmt::Error)?;
        Debug::fmt(&xml, f)
    }
}

#[derive(Debug)]
pub struct GdalWarpAppOptions(NonNull<GDALWarpAppOptions>);

impl GdalWarpAppOptions {
    /// Instatiate and empty set of warp options.
    pub fn new() -> Self {
        unsafe { Self::from_ptr(GDALWarpAppOptionsNew(ptr::null_mut(), ptr::null_mut())) }
    }

    /// Create Self from a raw pointer.
    ///
    /// # Safety
    /// Caller is responsible for ensuring `ptr` is not null, and
    /// ownership of `ptr` is properly transferred
    pub unsafe fn from_ptr(ptr: *mut GDALWarpAppOptions) -> Self {
        Self(NonNull::new_unchecked(ptr))
    }

    /// Get a immutable pointer to C API options.
    pub fn as_ptr(&self) -> *const GDALWarpAppOptions {
        self.0.as_ptr()
    }

    /// Get a mutable pointer to C API options.
    pub fn as_ptr_mut(&mut self) -> *mut GDALWarpAppOptions {
        self.0.as_ptr()
    }
}

impl Drop for GdalWarpAppOptions {
    fn drop(&mut self) {
        unsafe { GDALWarpAppOptionsFree(self.as_ptr_mut()) }
    }
}

impl Default for GdalWarpAppOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Specifies the initial value cells in the destination dataset during a warp operation.
///
/// See [`GdalWarpOptions::with_initial_value`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InitValue {
    NoData,
    Value(f64),
}

impl From<f64> for InitValue {
    fn from(v: f64) -> Self {
        InitValue::Value(v)
    }
}

impl Display for InitValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            InitValue::NoData => "NO_DATA".to_string(),
            InitValue::Value(v) => v.to_string(),
        };
        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::Result;
    use crate::raster::warp::*;
    use crate::raster::GdalDataType;

    fn ops_str(ops: &GdalWarpOptions) -> String {
        let xml = ops.to_xml().unwrap();
        xml.to_string()
    }

    #[test]
    fn defaults() {
        let opts = GdalWarpOptions::default();
        assert!(ops_str(&opts).contains("NearestNeighbour"));
    }

    #[test]
    fn with_settings() -> Result<()> {
        let mut opts = GdalWarpOptions::default();
        assert_eq!(opts.memory_limit(), 0);
        opts.with_memory_limit(1 << 16)
            .with_working_datatype(GdalDataType::UInt16)
            .with_band_count(2)
            .with_resampling_alg(WarpResampleAlg::Cubic);

        let extra = opts.extra_options_mut();
        extra.set_name_value("NUM_THREADS", "4")?;
        extra.set_name_value("SOURCE_EXTRA", "2")?;

        assert_eq!(opts.memory_limit(), 1 << 16);

        let expected = r#"<GDALWarpOptions>
  <WarpMemoryLimit>65536</WarpMemoryLimit>
  <ResampleAlg>Cubic</ResampleAlg>
  <WorkingDataType>UInt16</WorkingDataType>
  <Option name="NUM_THREADS">4</Option>
  <Option name="SOURCE_EXTRA">2</Option>
  <BandList>
    <BandMapping src="1" dst="1" />
    <BandMapping src="2" dst="2" />
  </BandList>
</GDALWarpOptions>"#;
        assert_eq!(ops_str(&opts), expected);

        Ok(())
    }

    #[test]
    fn band_count() -> Result<()> {
        let mut opts = GdalWarpOptions::default();
        assert_eq!(opts.band_count(), 0);
        opts.with_band_count(3);
        assert_eq!(opts.band_count(), 3);

        assert!(!ops_str(&opts).contains("<SrcNoDataReal>"));

        opts.apply_src_nodata(255.0)?;

        assert!(ops_str(&opts).contains("<SrcNoDataReal>255</SrcNoDataReal>"));

        opts.apply_dst_nodata(0.0)?;

        assert!(ops_str(&opts).contains("<DstNoDataReal>0</DstNoDataReal>"));

        Ok(())
    }

    #[test]
    fn init_value() -> Result<()> {
        let rendered = |v: &str| format!("<Option name=\"INIT_DEST\">{v}</Option>");

        let mut opts = GdalWarpOptions::default();
        assert_eq!(opts.initial_value(), None);

        opts.with_initial_value(InitValue::NoData);
        assert!(ops_str(&opts).contains(&rendered("NO_DATA")));

        opts.with_initial_value(InitValue::Value(255.0));
        assert!(ops_str(&opts).contains(&rendered("255")));

        Ok(())
    }
}
