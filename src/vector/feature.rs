use std::ffi::CString;
use libc::{c_void, c_double};
use vector::Defn;
use utils::_string;
use gdal_sys::{ogr, ogr_enums};
use vector::geometry::Geometry;
use vector::layer::Layer;
use gdal_major_object::MajorObject;

use errors::*;

/// OGR Feature
pub struct Feature<'a> {
    _defn: &'a Defn,
    c_feature: *const c_void,
    geometry: Geometry,
}


impl<'a> Feature<'a> {
    pub fn new(defn: &'a Defn) -> Feature {
        let c_feature = unsafe { ogr::OGR_F_Create(defn.c_defn()) };
            unsafe { Feature {
                     _defn: defn,
                     c_feature: c_feature,
                     geometry: Geometry::lazy_feature_geometry(),
                    } }
    }

    pub unsafe fn _with_c_feature(defn: &'a Defn, c_feature: *const c_void) -> Feature {
        return Feature{
            _defn: defn,
            c_feature: c_feature,
            geometry: Geometry::lazy_feature_geometry(),
        };
    }

    /// Get the value of a named field. If the field exists, it returns a
    /// `FieldValue` wrapper, that you need to unpack to a base type
    /// (string, float, etc). If the field is missing, returns `None`.
    pub fn field(&self, name: &str) -> Option<FieldValue> {
        let c_name = CString::new(name.as_bytes()).unwrap();
        let field_id = unsafe { ogr::OGR_F_GetFieldIndex(self.c_feature, c_name.as_ptr()) };
        if field_id == -1 {
            return None;
        }
        let field_defn = unsafe { ogr::OGR_F_GetFieldDefnRef(self.c_feature, field_id) };
        let field_type = unsafe { ogr::OGR_Fld_GetType(field_defn) };
        match field_type {
            ogr::OFT_STRING => {
                let rv = unsafe { ogr::OGR_F_GetFieldAsString(self.c_feature, field_id) };
                return Some(FieldValue::StringValue(_string(rv)));
            },
            ogr::OFT_REAL => {
                let rv = unsafe { ogr::OGR_F_GetFieldAsDouble(self.c_feature, field_id) };
                return Some(FieldValue::RealValue(rv as f64));
            },
            _ => panic!("Unknown field type {}", field_type)
        }
    }

    /// Get the field's geometry.
    pub fn geometry(&self) -> &Geometry {
        if ! self.geometry.has_gdal_ptr() {
            let c_geom = unsafe { ogr::OGR_F_GetGeometryRef(self.c_feature) };
            unsafe { self.geometry.set_c_geometry(c_geom) };
        }
        return &self.geometry;
    }

    pub fn create(&self, lyr: &Layer) -> Result<()> {
        let rv = unsafe { ogr::OGR_L_CreateFeature(lyr.gdal_object_ptr(), self.c_feature) };
        // assert_eq!(rv, ogr::OGRERR_NONE);
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_L_CreateFeature").into());
        }
        Ok(())
    }

    pub fn set_field_string(&self, field_name: &str, value: &str){
        let c_str_field_name = CString::new(field_name).unwrap();
        let c_str_value = CString::new(value).unwrap();
        let idx = unsafe { ogr::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        unsafe { ogr::OGR_F_SetFieldString(self.c_feature, idx, c_str_value.as_ptr()) };
    }

    pub fn set_field_double(&self, field_name: &str, value: f64){
        let c_str_field_name = CString::new(field_name).unwrap();
        let idx = unsafe { ogr::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        unsafe { ogr::OGR_F_SetFieldDouble(self.c_feature, idx, value as c_double) };
    }

    pub fn set_geometry(&mut self, geom: Geometry) -> Result<()> {
        self.geometry = geom;
        let rv = unsafe { ogr::OGR_F_SetGeometry(self.c_feature, self.geometry.c_geometry()) };
        if rv != ogr_enums::OGRErr::OGRERR_NONE {
            return Err(ErrorKind::OgrError(rv, "OGR_G_SetGeometry").into());
        }
        Ok(())
    }
}


impl<'a> Drop for Feature<'a> {
    fn drop(&mut self) {
        unsafe { ogr::OGR_F_Destroy(self.c_feature); }
    }
}


pub enum FieldValue {
    StringValue(String),
    RealValue(f64),
}


impl FieldValue {
    /// Interpret the value as `String`. Panics if the value is something else.
    pub fn as_string(self) -> String {
        match self {
            FieldValue::StringValue(rv) => rv,
            _ => panic!("not a StringValue")
        }
    }

    /// Interpret the value as `f64`. Panics if the value is something else.
    pub fn as_real(self) -> f64 {
        match self {
            FieldValue::RealValue(rv) => rv,
            _ => panic!("not a RealValue")
        }
    }
}
