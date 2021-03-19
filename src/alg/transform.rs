use core::f64;
use std::{
    cell::RefCell,
    ffi::{CStr, CString},
};

use libc::c_void;

use crate::{
    errors::{GdalError, Result},
    Dataset,
};

/// Ground Control Point (GCP) used to georeference a dataset.
#[derive(Debug, Clone)]
pub struct GCP {
    id: String,
    info: Option<String>,
    pixel_xy: (usize, usize),
    location_xyz: (f64, f64, f64),
}

impl GCP {
    /// Returns a GCP constructed from its GDAL C equivalent.
    ///
    /// # Arguments
    ///
    /// * `c_gcp` - A valid pointer to a C GDAL GCP representation.
    ///
    /// # Safety
    ///
    /// The pointer specified by `c_gcp` must point to a valid memory location.
    pub unsafe fn with_c_gcp(c_gcp: *const gdal_sys::GDAL_GCP) -> GCP {
        GCP {
            id: CStr::from_ptr((*c_gcp).pszId).to_str().unwrap().to_string(),
            info: CStr::from_ptr((*c_gcp).pszInfo)
                .to_str()
                .map_or(None, |str| Some(str.to_string())),
            pixel_xy: ((*c_gcp).dfGCPPixel as usize, (*c_gcp).dfGCPLine as usize),
            location_xyz: ((*c_gcp).dfGCPX, (*c_gcp).dfGCPY, (*c_gcp).dfGCPZ),
        }
    }
}

/// Polynomial order.
#[derive(Copy, Clone)]
pub enum Order {
    First = 1,
    Second = 2,
    Third = 3,
}
/// Transformer used to map between pixel/line/height to longitude/latitude/height.
pub struct Transformer {
    c_transformer_ref: RefCell<*mut c_void>,
}

impl Transformer {
    /// Construct a ```Transformer``` from a valid C GDAL pointer to a transformer instance.
    pub(crate) unsafe fn with_c_transformer(c_transformer: *mut c_void) -> Transformer {
        Transformer {
            c_transformer_ref: RefCell::new(c_transformer),
        }
    }

    /// Extracts the Ground Control Points from the specified dataset.
    ///
    /// # Arguments
    ///
    /// * `dataset` - The `Dataset` to extract Ground Control Points from.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use gdal::Dataset;
    /// use gdal::alg::transform::Transformer;
    ///
    /// let path = Path::new("/path/to/dataset");
    /// let dataset = Dataset::open(&path).unwrap();
    /// let gcps = Transformer::gcps(&dataset);
    /// ```
    pub fn gcps(dataset: &Dataset) -> Vec<GCP> {
        unsafe {
            let count = gdal_sys::GDALGetGCPCount(dataset.c_dataset()) as usize;
            let gcps = gdal_sys::GDALGetGCPs(dataset.c_dataset());
            let mut result = Vec::with_capacity(count);
            for i in 0..count {
                let gcp = GCP::with_c_gcp(gcps.add(i));
                result.push(gcp);
            }
            result
        }
    }

    /// Constructs a GCP based polynomial `Transformer`.
    ///
    /// # Arguments
    ///
    /// * `gcp`    - A vector of Ground Control Points to be used as input.
    /// * `order`  - The requested polynomial order. Using a third-order polynomial is not recommended due
    ///              to the potential for numeric instabilities.
    /// * `reversed` - Set it to `true` to compute the reversed transformation.
    pub fn gcp(gcp: Vec<GCP>, order: Order, reversed: bool) -> Result<Transformer> {
        let pas_gcp_list = gcp
            .iter()
            .map(|p| {
                let psz_info = match &p.info {
                    Some(arg) => {
                        let tmp = CString::new(arg.clone()).unwrap();
                        tmp.into_raw()
                    }
                    None => std::ptr::null_mut(),
                };

                gdal_sys::GDAL_GCP {
                    pszId: CString::new(p.id.clone()).unwrap().into_raw(),
                    pszInfo: psz_info,
                    dfGCPPixel: p.pixel_xy.0 as f64,
                    dfGCPLine: p.pixel_xy.1 as f64,
                    dfGCPX: p.location_xyz.0,
                    dfGCPY: p.location_xyz.1,
                    dfGCPZ: p.location_xyz.2,
                }
            })
            .collect::<Vec<_>>();

        let c_transformer = unsafe {
            gdal_sys::GDALCreateGCPTransformer(
                gcp.len() as i32,
                pas_gcp_list.as_ptr(),
                order as i32,
                if reversed { 1 } else { 0 },
            )
        };

        if c_transformer.is_null() {
            return Err(GdalError::NullPointer {
                method_name: "GDALCreateGCPTransformer",
                msg: "Failed to create GCP Transformer".to_string(),
            });
        }

        Ok(unsafe { Transformer::with_c_transformer(c_transformer) })
    }

    /// Transform a 2D point based on GCP derived polynomial model.
    ///
    /// # Arguments
    ///
    /// * `x` - The x point (pixel or longitude).
    /// * `y` - The y point (pixel or longitude).
    pub fn transform(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        let mut x: [f64; 1] = [x];
        let mut y: [f64; 1] = [y];
        let mut z: [f64; 1] = [0.];
        let mut r: [i32; 1] = [0];
        unsafe {
            gdal_sys::GDALGCPTransform(
                *self.c_transformer_ref.borrow(),
                0,
                1,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                z.as_mut_ptr(),
                r.as_mut_ptr(),
            );
            if r[0] != 0 {
                Some((x[0], y[0]))
            } else {
                None
            }
        }
    }
}

/// Cleanup the GCP transformer when it exits scope.
impl Drop for Transformer {
    fn drop(&mut self) {
        unsafe {
            let c_transformer = self.c_transformer_ref.borrow();
            gdal_sys::GDALDestroyGCPTransformer(c_transformer.to_owned());
        }
    }
}
