use std::ffi::c_double;
use std::mem::MaybeUninit;

use crate::errors;
use crate::errors::GdalError;

/// An affine transform.
///
/// A six-element array storing the coefficients of an [affine transform]
/// used in mapping coordinates between pixel/line `(P, L)` (raster) space,
/// and `(Xp,Yp)` (projection/[`crate::spatial_ref::SpatialRef`]) space.
///
/// # Interpretation
///
/// A `GeoTransform`'s components have the following meanings:
///
///   * `GeoTransform[0]`: x-coordinate of the upper-left corner of the upper-left pixel.
///   * `GeoTransform[1]`: W-E pixel resolution (pixel width).
///   * `GeoTransform[2]`: row rotation (typically zero).
///   * `GeoTransform[3]`: y-coordinate of the upper-left corner of the upper-left pixel.
///   * `GeoTransform[4]`: column rotation (typically zero).
///   * `GeoTransform[5]`: N-S pixel resolution (pixel height), negative value for a North-up image.
///
///
/// ## Note
///
/// Care with coefficient ordering is required when constructing an [affine transform matrix] from
/// a `GeoTransform`. If a 3x3 transform matrix is defined as:
///
/// ```text
/// | a b c |
/// | d e f |
/// | 0 0 1 |
/// ```
///
/// The corresponding `GeoTransform` ordering is:
///
/// ```text
/// [c, a, b, f, d, e]
/// ```
///
/// # Usage
///  *  [`apply`](GeoTransformEx::apply): perform a `(P,L) -> (Xp,Yp)` transformation
///  *  [`invert`](GeoTransformEx::invert):  construct the inverse transformation coefficients
///     for computing `(Xp,Yp) -> (P,L)` transformations
///
/// # Example
///
/// ```rust, no_run
/// # fn main() -> gdal::errors::Result<()> {
/// use gdal::{Dataset, GeoTransformEx};
/// let ds = Dataset::open("fixtures/m_3607824_se_17_1_20160620_sub.tif")?;
/// let transform = ds.geo_transform()?;
/// let (p, l) = (0.0, 0.0);
/// let (x,y) = transform.apply(p, l);
/// println!("(x,y): ({x},{y})");
/// let inverse = transform.invert()?;
/// let (p, l) = inverse.apply(x, y);
/// println!("(p,l): ({p},{l})");
/// # Ok(())
/// # }
/// ```
/// Output:
///
/// ```text
/// (x,y): (768269,4057292)
/// (p,l): (0,0)
/// ```
/// # See Also
///
///   * [GDAL GeoTransform Tutorial]
///   * [GDALGetGeoTransform]
///   * [Raster Data Model Affine Transform]
///
/// [GDAL GeoTransform Tutorial]: https://gdal.org/tutorials/geotransforms_tut.html
/// [GDALGetGeoTransform]: https://gdal.org/api/gdaldataset_cpp.html#classGDALDataset_1a5101119705f5fa2bc1344ab26f66fd1d
/// [Raster Data Model Affine Transform]: https://gdal.org/user/raster_data_model.html#affine-geotransform
/// [affine transform]: https://en.wikipedia.org/wiki/Affine_transformation
/// [affine transform matrix]: https://en.wikipedia.org/wiki/Transformation_matrix#Affine_transformations
pub type GeoTransform = [c_double; 6];

/// Extension methods on [`GeoTransform`]
pub trait GeoTransformEx {
    /// Apply GeoTransform to x/y coordinate.
    ///
    /// Wraps [GDALApplyGeoTransform].
    ///
    /// # Example
    ///
    /// See [`GeoTransform`](GeoTransform#example)
    ///
    /// [GDALApplyGeoTransform]: https://gdal.org/api/raster_c_api.html#_CPPv421GDALApplyGeoTransformPdddPdPd
    fn apply(&self, pixel: f64, line: f64) -> (f64, f64);

    /// Invert a [`GeoTransform`].
    ///
    /// Wraps [GDALInvGeoTransform].
    ///
    /// # Example
    ///
    /// See [`GeoTransform`](GeoTransform#example)
    ///
    /// [GDALInvGeoTransform]: https://gdal.org/api/raster_c_api.html#_CPPv419GDALInvGeoTransformPdPd
    fn invert(&self) -> errors::Result<GeoTransform>;
}

impl GeoTransformEx for GeoTransform {
    fn apply(&self, pixel: f64, line: f64) -> (f64, f64) {
        let mut geo_x = MaybeUninit::<f64>::uninit();
        let mut geo_y = MaybeUninit::<f64>::uninit();
        unsafe {
            gdal_sys::GDALApplyGeoTransform(
                self.as_ptr() as *mut f64,
                pixel,
                line,
                geo_x.as_mut_ptr(),
                geo_y.as_mut_ptr(),
            );
            (geo_x.assume_init(), geo_y.assume_init())
        }
    }

    fn invert(&self) -> errors::Result<GeoTransform> {
        let mut gt_out = MaybeUninit::<GeoTransform>::uninit();
        let rv = unsafe {
            gdal_sys::GDALInvGeoTransform(
                self.as_ptr() as *mut f64,
                (*gt_out.as_mut_ptr()).as_mut_ptr(),
            )
        };
        if rv == 0 {
            return Err(GdalError::BadArgument(
                "Geo transform is uninvertible".to_string(),
            ));
        }
        let result = unsafe { gt_out.assume_init() };
        Ok(result)
    }
}
