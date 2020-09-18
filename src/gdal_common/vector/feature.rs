use crate::utils::{_last_null_pointer_err, _string};
use crate::{Defn, Geometry, Layer};
use gdal_sys::{self, OGRErr, OGRFeatureH, OGRFieldType};
use libc::{c_double, c_int};
use std::ffi::CString;

#[cfg(feature = "datetime")]
use chrono::{Date, DateTime, Datelike, FixedOffset, TimeZone, Timelike};

use crate::errors::*;

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
        let geom_field_count =
            unsafe { gdal_sys::OGR_FD_GetGeomFieldCount(defn.c_defn()) } as isize;
        (0..geom_field_count)
            .map(|_| unsafe { Geometry::lazy_feature_geometry() })
            .collect()
    }

    /// Get the value of a named field. If the field exists, it returns a
    /// `FieldValue` wrapper, that you need to unpack to a base type
    /// (string, float, etc). If the field is missing, returns `None`.
    pub fn field(&self, name: &str) -> Result<FieldValue> {
        let c_name = CString::new(name)?;
        let field_id = unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_name.as_ptr()) };
        if field_id == -1 {
            Err(ErrorKind::InvalidFieldName {
                field_name: name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            })?;
        }
        let field_defn = unsafe { gdal_sys::OGR_F_GetFieldDefnRef(self.c_feature, field_id) };
        let field_type = unsafe { gdal_sys::OGR_Fld_GetType(field_defn) };
        match field_type {
            OGRFieldType::OFTString => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsString(self.c_feature, field_id) };
                Ok(FieldValue::StringValue(_string(rv)))
            }
            OGRFieldType::OFTReal => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.c_feature, field_id) };
                Ok(FieldValue::RealValue(rv as f64))
            }
            OGRFieldType::OFTInteger => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.c_feature, field_id) };
                Ok(FieldValue::IntegerValue(rv as i32))
            }
            #[cfg(feature = "datetime")]
            OGRFieldType::OFTDateTime => Ok(FieldValue::DateTimeValue(
                self.get_field_datetime(field_id)?,
            )),
            #[cfg(feature = "datetime")]
            OGRFieldType::OFTDate => Ok(FieldValue::DateValue(
                self.get_field_datetime(field_id)?.date(),
            )),
            _ => Err(ErrorKind::UnhandledFieldType {
                field_type,
                method_name: "OGR_Fld_GetType",
            })?,
        }
    }

    #[cfg(feature = "datetime")]
    fn get_field_datetime(&self, field_id: c_int) -> Result<DateTime<FixedOffset>> {
        let mut year: c_int = 0;
        let mut month: c_int = 0;
        let mut day: c_int = 0;
        let mut hour: c_int = 0;
        let mut minute: c_int = 0;
        let mut second: c_int = 0;
        let mut tzflag: c_int = 0;

        let success = unsafe {
            gdal_sys::OGR_F_GetFieldAsDateTime(
                self.c_feature,
                field_id,
                &mut year,
                &mut month,
                &mut day,
                &mut hour,
                &mut minute,
                &mut second,
                &mut tzflag,
            )
        };
        if success == 0 {
            Err(ErrorKind::OgrError {
                err: OGRErr::OGRERR_FAILURE,
                method_name: "OGR_F_GetFieldAsDateTime",
            })?;
        }

        // from https://github.com/OSGeo/gdal/blob/33a8a0edc764253b582e194d330eec3b83072863/gdal/ogr/ogrutils.cpp#L1309
        let tzoffset_secs = if tzflag == 0 || tzflag == 100 {
            0
        } else {
            (tzflag as i32 - 100) * 15 * 60
        };
        let rv = FixedOffset::east(tzoffset_secs)
            .ymd(year as i32, month as u32, day as u32)
            .and_hms(hour as u32, minute as u32, second as u32);
        Ok(rv)
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
        let idx =
            unsafe { gdal_sys::OGR_F_GetGeomFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "geometry_by_name",
            })?
        } else {
            self.geometry_by_index(idx as usize)
        }
    }

    pub fn geometry_by_index(&self, idx: usize) -> Result<&Geometry> {
        if idx >= self.geometry.len() {
            Err(ErrorKind::InvalidFieldIndex {
                index: idx,
                method_name: "geometry_by_name",
            })?;
        }
        if !self.geometry[idx].has_gdal_ptr() {
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
            Err(ErrorKind::OgrError {
                err: rv,
                method_name: "OGR_L_CreateFeature",
            })?;
        }
        Ok(())
    }

    pub fn set_field_string(&self, field_name: &str, value: &str) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let c_str_value = CString::new(value)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            })?;
        }
        unsafe { gdal_sys::OGR_F_SetFieldString(self.c_feature, idx, c_str_value.as_ptr()) };
        Ok(())
    }

    pub fn set_field_double(&self, field_name: &str, value: f64) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            })?;
        }
        unsafe { gdal_sys::OGR_F_SetFieldDouble(self.c_feature, idx, value as c_double) };
        Ok(())
    }

    pub fn set_field_integer(&self, field_name: &str, value: i32) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            })?;
        }
        unsafe { gdal_sys::OGR_F_SetFieldInteger(self.c_feature, idx, value as c_int) };
        Ok(())
    }

    #[cfg(feature = "datetime")]
    pub fn set_field_datetime(&self, field_name: &str, value: DateTime<FixedOffset>) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            Err(ErrorKind::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            })?;
        }

        let year = value.year() as c_int;
        let month = value.month() as c_int;
        let day = value.day() as c_int;
        let hour = value.hour() as c_int;
        let minute = value.minute() as c_int;
        let second = value.second() as c_int;
        let tzflag: c_int = if value.offset().local_minus_utc() == 0 {
            0
        } else {
            100 + (value.offset().local_minus_utc() / (15 * 60))
        };

        unsafe {
            gdal_sys::OGR_F_SetFieldDateTime(
                self.c_feature,
                idx,
                year,
                month,
                day,
                hour,
                minute,
                second,
                tzflag,
            )
        };
        Ok(())
    }

    pub fn set_field(&self, field_name: &str, value: &FieldValue) -> Result<()> {
        match *value {
            FieldValue::RealValue(value) => self.set_field_double(field_name, value),
            FieldValue::StringValue(ref value) => self.set_field_string(field_name, value.as_str()),
            FieldValue::IntegerValue(value) => self.set_field_integer(field_name, value),

            #[cfg(feature = "datetime")]
            FieldValue::DateTimeValue(value) => self.set_field_datetime(field_name, value),

            #[cfg(feature = "datetime")]
            FieldValue::DateValue(value) => {
                self.set_field_datetime(field_name, value.and_hms(0, 0, 0))
            }
        }
    }

    pub fn set_geometry(&mut self, geom: Geometry) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_F_SetGeometry(self.c_feature, geom.c_geometry()) };
        if rv != OGRErr::OGRERR_NONE {
            Err(ErrorKind::OgrError {
                err: rv,
                method_name: "OGR_G_SetGeometry",
            })?;
        }
        self.geometry[0] = geom;
        Ok(())
    }
}

impl<'a> Drop for Feature<'a> {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::OGR_F_Destroy(self.c_feature);
        }
    }
}

pub enum FieldValue {
    IntegerValue(i32),
    StringValue(String),
    RealValue(f64),

    #[cfg(feature = "datetime")]
    DateValue(Date<FixedOffset>),

    #[cfg(feature = "datetime")]
    DateTimeValue(DateTime<FixedOffset>),
}

impl FieldValue {
    /// Interpret the value as `String`. Panics if the value is something else.
    pub fn into_string(self) -> Option<String> {
        match self {
            FieldValue::StringValue(rv) => Some(rv),
            _ => None,
        }
    }

    /// Interpret the value as `f64`. Panics if the value is something else.
    pub fn into_real(self) -> Option<f64> {
        match self {
            FieldValue::RealValue(rv) => Some(rv),
            _ => None,
        }
    }

    /// Interpret the value as `i32`. Panics if the value is something else.
    pub fn into_int(self) -> Option<i32> {
        match self {
            FieldValue::IntegerValue(rv) => Some(rv),
            _ => None,
        }
    }

    /// Interpret the value as `Date`.
    #[cfg(feature = "datetime")]
    pub fn into_date(self) -> Option<Date<FixedOffset>> {
        match self {
            FieldValue::DateValue(rv) => Some(rv),
            FieldValue::DateTimeValue(rv) => Some(rv.date()),
            _ => None,
        }
    }

    /// Interpret the value as `DateTime`.
    #[cfg(feature = "datetime")]
    pub fn into_datetime(self) -> Option<DateTime<FixedOffset>> {
        match self {
            FieldValue::DateTimeValue(rv) => Some(rv),
            _ => None,
        }
    }
}
