use crate::errors;
use crate::errors::GdalError;
use crate::spatial_ref::{CoordTransformOptions, SpatialRef};
use crate::utils::{_last_cpl_err, _last_null_pointer_err};
use gdal_sys::{CPLErr, OGRCoordinateTransformationH};
use libc::c_int;
use std::ptr::null_mut;

#[derive(Debug)]
/// Defines a coordinate transformation from one [`SpatialRef`] to another.
pub struct CoordTransform {
    inner: OGRCoordinateTransformationH,
    from: String,
    to: String,
}

impl Drop for CoordTransform {
    fn drop(&mut self) {
        unsafe { gdal_sys::OCTDestroyCoordinateTransformation(self.inner) };
    }
}

impl CoordTransform {
    /// Constructs a new transformation from `source` to `target`.
    ///
    /// See: [OCTNewCoordinateTransformation](https://gdal.org/api/ogr_srs_api.html#_CPPv430OCTNewCoordinateTransformation20OGRSpatialReferenceH20OGRSpatialReferenceH)
    pub fn new(source: &SpatialRef, target: &SpatialRef) -> errors::Result<CoordTransform> {
        let c_obj = unsafe {
            gdal_sys::OCTNewCoordinateTransformation(source.to_c_hsrs(), target.to_c_hsrs())
        };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OCTNewCoordinateTransformation"));
        }
        Ok(Self {
            inner: c_obj,
            from: source.authority().or_else(|_| source.to_proj4())?,
            to: target.authority().or_else(|_| target.to_proj4())?,
        })
    }

    /// Constructs a new transformation from `source` to `target` with additional extended options
    /// defined by `options`: [`CoordTransformOptions`].
    ///
    /// See: [OCTNewCoordinateTransformation](https://gdal.org/api/ogr_srs_api.html#_CPPv432OCTNewCoordinateTransformationEx20OGRSpatialReferenceH20OGRSpatialReferenceH35OGRCoordinateTransformationOptionsH)
    pub fn new_with_options(
        source: &SpatialRef,
        target: &SpatialRef,
        options: &CoordTransformOptions,
    ) -> errors::Result<CoordTransform> {
        let c_obj = unsafe {
            gdal_sys::OCTNewCoordinateTransformationEx(
                source.to_c_hsrs(),
                target.to_c_hsrs(),
                options.c_options(),
            )
        };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OCTNewCoordinateTransformation"));
        }
        Ok(Self {
            inner: c_obj,
            from: source.authority().or_else(|_| source.to_proj4())?,
            to: target.authority().or_else(|_| target.to_proj4())?,
        })
    }

    /// Transform bounding box, densifying the edges to account for nonlinear
    /// transformations.
    ///
    /// # Arguments
    /// * `bounds` - array of [axis0_min, axis1_min, axis0_max, axis1_max],
    ///              interpreted in the axis order of the source SpatialRef,
    ///              typically [xmin, ymin, xmax, ymax]
    /// * `densify_pts` - number of points per edge (recommended: 21)
    ///
    /// # Returns
    /// `Ok([f64; 4])` with bounds in axis order of target SpatialRef
    /// `Err` if there is an error.
    ///
    /// See: [OCTTransformBounds](https://gdal.org/api/ogr_srs_api.html#_CPPv418OCTTransformBounds28OGRCoordinateTransformationHKdKdKdKdPdPdPdPdKi)
    #[cfg(all(major_ge_3, minor_ge_4))]
    pub fn transform_bounds(
        &self,
        bounds: &[f64; 4],
        densify_pts: i32,
    ) -> errors::Result<[f64; 4]> {
        let mut out_xmin: f64 = 0.;
        let mut out_ymin: f64 = 0.;
        let mut out_xmax: f64 = 0.;
        let mut out_ymax: f64 = 0.;

        let ret_val = unsafe {
            gdal_sys::OCTTransformBounds(
                self.inner,
                bounds[0],
                bounds[1],
                bounds[2],
                bounds[3],
                &mut out_xmin,
                &mut out_ymin,
                &mut out_xmax,
                &mut out_ymax,
                densify_pts as c_int,
            ) == 1
        };

        if !ret_val {
            let msg = match _last_cpl_err(CPLErr::CE_Failure) {
                GdalError::CplError { msg, .. } => match msg.is_empty() {
                    false => Some(msg),
                    _ => None,
                },
                err => return Err(err),
            };
            return Err(GdalError::InvalidCoordinateRange {
                from: self.from.clone(),
                to: self.to.clone(),
                msg,
            });
        }

        Ok([out_xmin, out_ymin, out_xmax, out_ymax])
    }

    /// Transform coordinates in place.
    ///
    /// # Arguments
    /// * `x` - slice of x coordinates
    /// * `y` - slice of y coordinates (must match x in length)
    /// * `z` - slice of z coordinates, or an empty slice to ignore
    ///
    /// See: [OCTTransform](https://gdal.org/api/ogr_srs_api.html#_CPPv412OCTTransform28OGRCoordinateTransformationHiPdPdPd)
    pub fn transform_coords(
        &self,
        x: &mut [f64],
        y: &mut [f64],
        z: &mut [f64],
    ) -> errors::Result<()> {
        let nb_coords = x.len();
        assert_eq!(
            nb_coords,
            y.len(),
            "transform coordinate slices have different lengths: {} != {}",
            nb_coords,
            y.len()
        );
        let ret_val = unsafe {
            gdal_sys::OCTTransform(
                self.inner,
                nb_coords as c_int,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                if z.is_empty() {
                    null_mut()
                } else {
                    assert_eq!(
                        nb_coords,
                        z.len(),
                        "transform coordinate slices have different lengths: {} != {}",
                        nb_coords,
                        z.len()
                    );
                    z.as_mut_ptr()
                },
            ) == 1
        };

        if ret_val {
            Ok(())
        } else {
            let err = _last_cpl_err(CPLErr::CE_Failure);
            let msg = if let GdalError::CplError { msg, .. } = err {
                if msg.trim().is_empty() {
                    None
                } else {
                    Some(msg)
                }
            } else {
                return Err(err);
            };
            Err(GdalError::InvalidCoordinateRange {
                from: self.from.clone(),
                to: self.to.clone(),
                msg,
            })
        }
    }

    #[deprecated(since = "0.3.1", note = "use `transform_coords` instead")]
    pub fn transform_coord(&self, x: &mut [f64], y: &mut [f64], z: &mut [f64]) {
        self.transform_coords(x, y, z)
            .expect("Coordinate transform failed")
    }

    /// Returns a C pointer to the allocated [`gdal_sys::OGRCoordinateTransformationH`] memory.
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn to_c_hct(&self) -> OGRCoordinateTransformationH {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_almost_eq;
    use crate::vector::Geometry;

    #[cfg(all(major_ge_3, minor_ge_4))]
    #[test]
    fn transform_bounds() {
        let bounds: [f64; 4] = [-180., -80., 180., 80.];
        // bounds for y,x ordered SpatialRefs
        let yx_bounds: [f64; 4] = [-80.0, -180.0, 80.0, 180.];

        let spatial_ref1 = SpatialRef::from_definition("OGC:CRS84").unwrap();

        // transforming between the same SpatialRef should return existing bounds
        let mut transform = CoordTransform::new(&spatial_ref1, &spatial_ref1).unwrap();
        let mut out_bounds = transform.transform_bounds(&bounds, 21).unwrap();
        assert_almost_eq(out_bounds[0], bounds[0]);
        assert_almost_eq(out_bounds[1], bounds[1]);
        assert_almost_eq(out_bounds[2], bounds[2]);
        assert_almost_eq(out_bounds[3], bounds[3]);

        // EPSG:4326 is in y,x order by default; returned bounds are [ymin, xmin, ymax, xmax]
        let mut spatial_ref2 = SpatialRef::from_epsg(4326).unwrap();
        transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
        out_bounds = transform.transform_bounds(&bounds, 21).unwrap();
        assert_almost_eq(out_bounds[0], yx_bounds[0]);
        assert_almost_eq(out_bounds[1], yx_bounds[1]);
        assert_almost_eq(out_bounds[2], yx_bounds[2]);
        assert_almost_eq(out_bounds[3], yx_bounds[3]);

        // if source SpatialRef is in y,x order and and target SpatialRef is in x,y order
        // input bounds are interpreted as [ymin, xmin, ymax, xmax] and returns
        // [xmin, ymin, xmax, ymax]
        transform = CoordTransform::new(&spatial_ref2, &spatial_ref1).unwrap();
        out_bounds = transform.transform_bounds(&yx_bounds, 21).unwrap();
        assert_almost_eq(out_bounds[0], bounds[0]);
        assert_almost_eq(out_bounds[1], bounds[1]);
        assert_almost_eq(out_bounds[2], bounds[2]);
        assert_almost_eq(out_bounds[3], bounds[3]);

        // force EPSG:4326 into x,y order to match source SpatialRef
        spatial_ref2.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
        out_bounds = transform.transform_bounds(&bounds, 21).unwrap();
        assert_almost_eq(out_bounds[0], bounds[0]);
        assert_almost_eq(out_bounds[1], bounds[1]);
        assert_almost_eq(out_bounds[2], bounds[2]);
        assert_almost_eq(out_bounds[3], bounds[3]);

        spatial_ref2 = SpatialRef::from_epsg(3857).unwrap();
        transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
        out_bounds = transform.transform_bounds(&bounds, 21).unwrap();

        let expected_bounds: [f64; 4] = [
            -20037508.342789244,
            -15538711.096309224,
            20037508.342789244,
            15538711.09630923,
        ];
        assert_almost_eq(out_bounds[0], expected_bounds[0]);
        assert_almost_eq(out_bounds[1], expected_bounds[1]);
        assert_almost_eq(out_bounds[2], expected_bounds[2]);
        assert_almost_eq(out_bounds[3], expected_bounds[3]);
    }

    #[test]
    fn transform_coordinates() {
        let spatial_ref1 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
        let spatial_ref2 = SpatialRef::from_epsg(3035).unwrap();

        // TODO: handle axis order in tests
        #[cfg(major_ge_3)]
        spatial_ref1.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        #[cfg(major_ge_3)]
        spatial_ref2.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );

        let transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
        let mut xs = [23.43, 23.50];
        let mut ys = [37.58, 37.70];
        let mut zs = [32.0, 20.0];
        transform
            .transform_coords(&mut xs, &mut ys, &mut zs)
            .unwrap();
        assert_almost_eq(xs[0], 5509543.1508097);
        assert_almost_eq(ys[0], 1716062.1916192223);
        assert_almost_eq(zs[0], 32.0);
    }

    #[test]
    fn transform_ogr_geometry() {
        //let expected_value = "POLYGON ((5509543.150809700600803 1716062.191619219258428,5467122.000330002978444 1980151.204280239529908,5623571.028492723591626 2010213.310253676958382,5671834.921544363722205 1746968.078280254499987,5509543.150809700600803 1716062.191619219258428))";
        //let expected_value = "POLYGON ((5509543.15080969966948 1716062.191619222285226,5467122.000330002047122 1980151.204280242323875,5623571.028492721728981 2010213.31025367998518,5671834.921544362790883 1746968.078280256595463,5509543.15080969966948 1716062.191619222285226))";
        let expected_value = "POLYGON ((5509543.1508097 1716062.19161922,5467122.00033 1980151.20428024,5623571.02849272 2010213.31025368,5671834.92154436 1746968.07828026,5509543.1508097 1716062.19161922))";
        let mut geom = Geometry::from_wkt(
            "POLYGON((23.43 37.58, 23.43 40.0, 25.29 40.0, 25.29 37.58, 23.43 37.58))",
        )
        .unwrap();
        let spatial_ref1 = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();
        let spatial_ref2 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();

        // TODO: handle axis order in tests
        #[cfg(major_ge_3)]
        spatial_ref1.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        #[cfg(major_ge_3)]
        spatial_ref2.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );

        let htransform = CoordTransform::new(&spatial_ref2, &spatial_ref1).unwrap();
        geom.transform_inplace(&htransform).unwrap();
        assert_eq!(expected_value, geom.wkt().unwrap());
    }

    #[test]
    fn failing_transformation() {
        let wgs84 = SpatialRef::from_epsg(4326).unwrap();
        let dhd_2 = SpatialRef::from_epsg(31462).unwrap();

        // TODO: handle axis order in tests
        #[cfg(major_ge_3)]
        wgs84.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        #[cfg(major_ge_3)]
        dhd_2.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );

        let mut x = [1979105.06, 0.0];
        let mut y = [5694052.67, 0.0];
        let mut z = [0.0, 0.0];

        let trafo = CoordTransform::new(&wgs84, &dhd_2).unwrap();
        let r = trafo.transform_coords(&mut x, &mut y, &mut z);
        assert!(r.is_err());

        let wgs84 = SpatialRef::from_epsg(4326).unwrap();
        let webmercator = SpatialRef::from_epsg(3857).unwrap();

        // TODO: handle axis order in tests
        #[cfg(major_ge_3)]
        wgs84.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );
        #[cfg(major_ge_3)]
        webmercator.set_axis_mapping_strategy(
            gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER,
        );

        let mut x = [1000000.0];
        let mut y = [1000000.0];

        let trafo = CoordTransform::new(&wgs84, &webmercator).unwrap();
        let r = trafo.transform_coords(&mut x, &mut y, &mut []);

        assert!(r.is_err());
        if let GdalError::InvalidCoordinateRange { .. } = r.unwrap_err() {
            // assert_eq!(msg, &Some("latitude or longitude exceeded limits".into()));
        } else {
            panic!("Wrong error type");
        }
    }
}
