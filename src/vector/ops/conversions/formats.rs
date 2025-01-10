use std::{
    ffi::{c_char, c_void, CString},
    ptr::null_mut,
};

use gdal_sys::OGRErr;

use crate::errors::GdalError;
use crate::errors::Result;
use crate::utils::{_last_null_pointer_err, _string};
use crate::vector::Geometry;

/// Methods supporting translation between GDAL [`Geometry`] and various text representations.
///
/// These include:
/// * ["Well Known" representations of geometry][wikipedia].
/// * [GeoJSON][geojson]
///
/// [wikipedia]: https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry
/// [geojson]: https://geojson.org/
///
impl Geometry {
    /// Create a geometry by parsing a
    /// [WKT](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry) string.
    pub fn from_wkt(wkt: &str) -> Result<Geometry> {
        let c_wkt = CString::new(wkt)?;
        // OGR_G_CreateFromWkt does not write to the pointed-to memory, but this is not reflected
        // in its signature (`char**` instead of `char const**`), so we need a scary looking cast.
        let mut c_wkt_ptr = c_wkt.as_ptr() as *mut c_char;
        let mut c_geom = null_mut();
        let rv = unsafe { gdal_sys::OGR_G_CreateFromWkt(&mut c_wkt_ptr, null_mut(), &mut c_geom) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_CreateFromWkt",
            });
        }
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Creates a geometry by parsing a slice of bytes in
    /// [WKB](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry#Well-known_binary)
    /// (Well-Known Binary) format.
    pub fn from_wkb(wkb: &[u8]) -> Result<Geometry> {
        let mut c_geom = null_mut();
        let rv = unsafe {
            gdal_sys::OGR_G_CreateFromWkb(
                wkb.as_ptr() as *const std::ffi::c_void,
                null_mut(),
                &mut c_geom,
                wkb.len() as i32,
            )
        };
        if rv != gdal_sys::OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_CreateFromWkb",
            });
        }
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Create a geometry by parsing a
    /// [GeoJSON](https://en.wikipedia.org/wiki/GeoJSON) string.
    pub fn from_geojson(json: &str) -> Result<Geometry> {
        let c_geojson = CString::new(json)?;
        let c_geom = unsafe { gdal_sys::OGR_G_CreateGeometryFromJson(c_geojson.as_ptr()) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_CreateGeometryFromJson"));
        }
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Create a geometry by parsing a
    /// [GML](https://en.wikipedia.org/wiki/Geography_Markup_Language) string.
    pub fn from_gml(json: &str) -> Result<Geometry> {
        let c_gml = CString::new(json)?;
        let c_geom = unsafe { gdal_sys::OGR_G_CreateFromGML(c_gml.as_ptr()) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_G_CreateFromGML"));
        }
        Ok(unsafe { Geometry::with_c_geometry(c_geom, true) })
    }

    /// Serialize the geometry as WKT.
    pub fn wkt(&self) -> Result<String> {
        let mut c_wkt = null_mut();
        let rv = unsafe { gdal_sys::OGR_G_ExportToWkt(self.c_geometry(), &mut c_wkt) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_ExportToWkt",
            });
        }
        let wkt = _string(c_wkt).unwrap_or_default();
        unsafe { gdal_sys::VSIFree(c_wkt as *mut c_void) };
        Ok(wkt)
    }

    /// Serialize the geometry as SFSQL 1.2 / ISO SQL / MM Part 3 WKT.
    pub fn iso_wkt(&self) -> Result<String> {
        let mut c_wkt = null_mut();
        let rv = unsafe { gdal_sys::OGR_G_ExportToIsoWkt(self.c_geometry(), &mut c_wkt) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_ExportToIsoWkt",
            });
        }
        let wkt = _string(c_wkt).unwrap_or_default();
        unsafe { gdal_sys::VSIFree(c_wkt as *mut c_void) };
        Ok(wkt)
    }

    /// Serializes the geometry to
    /// [WKB](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry#Well-known_binary)
    /// (Well-Known Binary) format.
    pub fn wkb(&self) -> Result<Vec<u8>> {
        let wkb_size = unsafe { gdal_sys::OGR_G_WkbSize(self.c_geometry()) as usize };
        // We default to little-endian for now. A WKB string explicitly indicates the byte
        // order, so this is not a problem for interoperability.
        // 
        // Consider using `Vec::MaybeUninit` in future.
        let byte_order = gdal_sys::OGRwkbByteOrder::wkbNDR;
        let mut wkb = vec![0; wkb_size];
        let rv =
            unsafe { gdal_sys::OGR_G_ExportToWkb(self.c_geometry(), byte_order, wkb.as_mut_ptr()) };
        if rv != gdal_sys::OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_ExportToWkb",
            });
        }
        Ok(wkb)
    }

    /// Serializes the geometry to SFSQL 1.2 / ISO SQL / MM Part 3
    /// [WKB](https://en.wikipedia.org/wiki/Well-known_text_representation_of_geometry#Well-known_binary)
    /// (Well-Known Binary) format.
    pub fn iso_wkb(&self) -> Result<Vec<u8>> {
        let wkb_size = unsafe { gdal_sys::OGR_G_WkbSize(self.c_geometry()) as usize };
        // We default to little-endian for now. A WKB string explicitly indicates the byte
        // order, so this is not a problem for interoperability.
        //
        // Consider using `Vec::MaybeUninit` in future.
        let byte_order = gdal_sys::OGRwkbByteOrder::wkbNDR;
        let mut wkb = vec![0; wkb_size];
        let rv =
            unsafe { gdal_sys::OGR_G_ExportToIsoWkb(self.c_geometry(), byte_order, wkb.as_mut_ptr()) };
        if rv != gdal_sys::OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_G_ExportToIsoWkb",
            });
        }
        Ok(wkb)
    }

    /// Serialize the geometry as GeoJSON.
    ///
    /// See: [`OGR_G_ExportToJson`](https://gdal.org/api/vector_c_api.html#_CPPv418OGR_G_ExportToJson12OGRGeometryH)
    pub fn json(&self) -> Result<String> {
        let c_json = unsafe { gdal_sys::OGR_G_ExportToJson(self.c_geometry()) };
        let rv = _string(c_json).ok_or_else(|| _last_null_pointer_err("OGR_G_ExportToJson"));
        unsafe { gdal_sys::VSIFree(c_json as *mut c_void) };
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_wkb() {
        let wkt = "POLYGON ((45.0 45.0, 45.0 50.0, 50.0 50.0, 50.0 45.0, 45.0 45.0))";
        let orig_geom = Geometry::from_wkt(wkt).unwrap();
        let wkb = orig_geom.wkb().unwrap();
        let new_geom = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(new_geom, orig_geom);
    }

    #[test]
    pub fn test_iso_wkb() {
        let wkt = "POLYGON ZM ((45.0 45.0 0.0 0.0, 45.0 50.0 0.0 0.25, 50.0 50.0 0.0 0.50, 50.0 45.0 0.0 0.75, 45.0 45.0 0.0 1.0))";
        let orig_geom = Geometry::from_wkt(wkt).unwrap();
        let wkb = orig_geom.iso_wkb().unwrap();
        let new_geom = Geometry::from_wkb(&wkb).unwrap();
        assert_eq!(new_geom, orig_geom);
    }

    #[test]
    pub fn test_geojson() {
        let json = r#"{ "type": "Point", "coordinates": [10, 20] }"#;
        let geom = Geometry::from_geojson(json).unwrap();
        let (x, y, _) = geom.get_point(0);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);

        let json = r#"{ "type": "Point", "coordinates": [10, 20 }"#;
        let res = Geometry::from_geojson(json);
        assert!(res.is_err());
    }

    #[test]
    pub fn test_gml() {
        let json = r#"<gml:Point xmlns:gml="http://www.opengis.net/gml"><gml:coordinates>10,20</gml:coordinates></gml:Point>"#;
        let geom = Geometry::from_gml(json).unwrap();
        let (x, y, _) = geom.get_point(0);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);

        let json = r#"<gml:Point xmlns:gml="http://www.opengis.net/gml"><gml:coordinates>10</gml:coordinates></gml:Point>"#;
        let res = Geometry::from_gml(json);
        assert!(res.is_err());
    }
}
