use std::ffi::CString;
use libc::{c_double, c_int};
use vector::Defn;
use utils::{_string, _last_null_pointer_err};
use gdal_sys::{self, OGRErr, OGRFeatureH, OGRFieldType};
use vector::geometry::Geometry;
use vector::layer::Layer;

use errors::*;

/// OGR Feature
pub struct Feature<'a> {
    _defn: &'a Defn,
    c_feature: OGRFeatureH,
    geometry: Vec<Geometry>,
}

impl<'a> Feature<'a> {
    pub fn new(defn: &'a Defn) -> Result<Feature> {
        let c_feature = unsafe { gdal_sys::OGR_F_Create(defn.c_defn()) };
        if c_feature.is_null() {
            Err(_last_null_pointer_err("OGR_F_Create"))?;
        };
        Ok(Feature {
                 _defn: defn,
                 c_feature,
                 geometry: Feature::_lazy_feature_geometries(defn),
             })
    }

    pub unsafe fn _with_c_feature(defn: &'a Defn, c_feature: OGRFeatureH) -> Feature {
        Feature {
            _defn: defn,
            c_feature,
            geometry: Feature::_lazy_feature_geometries(defn),
        }
    }

    pub fn _lazy_feature_geometries(defn: &'a Defn) -> Vec<Geometry> {
        let geom_field_count = unsafe { gdal_sys::OGR_FD_GetGeomFieldCount(defn.c_defn()) } as isize;
        (0..geom_field_count).map(|_| unsafe { Geometry::lazy_feature_geometry() }).collect()
    }

    /// Get the value of a named field. If the field exists, it returns a
    /// `FieldValue` wrapper, that you need to unpack to a base type
    /// (string, float, etc). If the field is missing, returns `None`.
    pub fn field(&self, name: &str) -> Result<FieldValue> {
        let c_name = CString::new(name)?;
        let field_id = unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_name.as_ptr()) };
        if field_id == -1 {
            Err(ErrorKind::InvalidFieldName{field_name: name.to_string(), method_name: "OGR_F_GetFieldIndex"})?;
        }
        let field_defn = unsafe { gdal_sys::OGR_F_GetFieldDefnRef(self.c_feature, field_id) };
        let field_type = unsafe { gdal_sys::OGR_Fld_GetType(field_defn) };
        match field_type {
            OGRFieldType::OFTString => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsString(self.c_feature, field_id) };
                Ok(FieldValue::StringValue(_string(rv)))
            },
            OGRFieldType::OFTReal => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.c_feature, field_id) };
                Ok(FieldValue::RealValue(rv as f64))
            },
            OGRFieldType::OFTInteger => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.c_feature, field_id) };
                Ok(FieldValue::IntegerValue(rv as i32))
            },
            _ => Err(ErrorKind::UnhandledFieldType{field_type, method_name: "OGR_Fld_GetType"})?
        }
    }

    /// Get the field's geometry.
    pub fn geometry(&self) -> &Geometry {
        if !self.geometry[0].has_gdal_ptr() {
            let c_geom = unsafe { gdal_sys::OGR_F_GetGeometryRef(self.c_feature) };
            unsafe { self.geometry[0].set_c_geometry(c_geom) };
        }
        &self.geometry[0]
    }

    pub fn geometry_by_name(&self, field_name: &str) -> Result<&Geometry> {
        let c_str_field_name = CString::new(field_name)?;
        let idx = unsafe { gdal_sys::OGR_F_GetGeomFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName{field_name: field_name.to_string(), method_name: "geometry_by_name"})?
        } else {
            self.geometry_by_index(idx as usize)
        }
    }

    pub fn geometry_by_index(&self, idx: usize) -> Result<&Geometry> {
        if idx >= self.geometry.len() {
            Err(ErrorKind::InvalidFieldIndex{index: idx, method_name: "geometry_by_name"})?;
        }
        if ! self.geometry[idx].has_gdal_ptr() {
            let c_geom = unsafe { gdal_sys::OGR_F_GetGeomFieldRef(self.c_feature, idx as i32) };
            if c_geom.is_null() {
                Err(_last_null_pointer_err("OGR_F_GetGeomFieldRef"))?;
            }
            unsafe { self.geometry[idx].set_c_geometry(c_geom) };
        }
        Ok(&self.geometry[idx])
    }

    pub fn create(&self, lyr: &Layer) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_L_CreateFeature(lyr.c_layer(), self.c_feature) };
        if rv != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError{err: rv, method_name: "OGR_L_CreateFeature"})?;
        }
        Ok(())
    }

    pub fn set_field_string(&self, field_name: &str, value: &str) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let c_str_value = CString::new(value)?;
        let idx = unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName{field_name: field_name.to_string(), method_name: "OGR_F_GetFieldIndex"})?;
        }
        unsafe { gdal_sys::OGR_F_SetFieldString(self.c_feature, idx, c_str_value.as_ptr()) };
        Ok(())
    }

    pub fn set_field_double(&self, field_name: &str, value: f64) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx = unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName{field_name: field_name.to_string(), method_name: "OGR_F_GetFieldIndex"})?;
        }
        unsafe { gdal_sys::OGR_F_SetFieldDouble(self.c_feature, idx, value as c_double) };
        Ok(())
    }

    pub fn set_field_integer(&self, field_name: &str, value: i32) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx = unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr())};
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName{field_name: field_name.to_string(), method_name: "OGR_F_GetFieldIndex"})?;
        }
        unsafe { gdal_sys::OGR_F_SetFieldInteger(self.c_feature, idx, value as c_int) };
        Ok(())
    }

    pub fn set_field(&self, field_name: &str,  value: &FieldValue) -> Result<()> {
          match *value {
             FieldValue::RealValue(value) => self.set_field_double(field_name, value),
             FieldValue::StringValue(ref value) => self.set_field_string(field_name, value.as_str()),
             FieldValue::IntegerValue(value) => self.set_field_integer(field_name, value)
         }
     }

    pub fn set_geometry(&mut self, geom: Geometry) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_F_SetGeometry(self.c_feature, geom.c_geometry()) };
        if rv != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError{err: rv, method_name: "OGR_G_SetGeometry"})?;
        }
        self.geometry[0] = geom;
        Ok(())
    }
}


impl<'a> Drop for Feature<'a> {
    fn drop(&mut self) {
        unsafe { gdal_sys::OGR_F_Destroy(self.c_feature); }
    }
}

pub enum FieldValue {
    IntegerValue(i32),
    StringValue(String),
    RealValue(f64),
}


impl FieldValue {
    /// Interpret the value as `String`. Panics if the value is something else.
    pub fn into_string(self) -> Option<String> {
        match self {
            FieldValue::StringValue(rv) => Some(rv),
            _ => None
        }
    }

    /// Interpret the value as `f64`. Panics if the value is something else.
    pub fn into_real(self) -> Option<f64> {
        match self {
            FieldValue::RealValue(rv) => Some(rv),
            _ => None
        }
    }

    /// Interpret the value as `i32`. Panics if the value is something else.
    pub fn into_int(self) -> Option<i32> {
        match self {
            FieldValue::IntegerValue(rv) => Some(rv),
            _ => None
        }
    }
}
