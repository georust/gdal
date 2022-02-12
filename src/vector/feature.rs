use crate::utils::{_last_null_pointer_err, _string, _string_array};
use crate::vector::geometry::Geometry;
use crate::vector::{Defn, LayerAccess};
use gdal_sys::{self, OGRErr, OGRFeatureH, OGRFieldType};
use libc::c_longlong;
use libc::{c_double, c_int};
use std::convert::TryInto;
use std::ffi::CString;

use chrono::{Date, DateTime, Datelike, FixedOffset, TimeZone, Timelike};

use crate::errors::*;
use std::slice;

/// OGR Feature
#[derive(Debug)]
pub struct Feature<'a> {
    _defn: &'a Defn,
    c_feature: OGRFeatureH,
    geometry: Vec<Geometry>,
}

impl<'a> Feature<'a> {
    pub fn new(defn: &'a Defn) -> Result<Feature> {
        let c_feature = unsafe { gdal_sys::OGR_F_Create(defn.c_defn()) };
        if c_feature.is_null() {
            return Err(_last_null_pointer_err("OGR_F_Create"));
        };
        Ok(Feature {
            _defn: defn,
            c_feature,
            geometry: Feature::_lazy_feature_geometries(defn),
        })
    }

    /// Creates a new Feature by wrapping a C pointer and a Defn
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_feature(defn: &'a Defn, c_feature: OGRFeatureH) -> Feature {
        Feature {
            _defn: defn,
            c_feature,
            geometry: Feature::_lazy_feature_geometries(defn),
        }
    }

    /// Returns the C wrapped pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_feature(&self) -> OGRFeatureH {
        self.c_feature
    }

    pub fn _lazy_feature_geometries(defn: &'a Defn) -> Vec<Geometry> {
        let geom_field_count =
            unsafe { gdal_sys::OGR_FD_GetGeomFieldCount(defn.c_defn()) } as isize;
        (0..geom_field_count)
            .map(|_| unsafe { Geometry::lazy_feature_geometry() })
            .collect()
    }

    /// Returns the feature identifier, or `None` if none has been assigned.
    pub fn fid(&self) -> Option<u64> {
        let fid = unsafe { gdal_sys::OGR_F_GetFID(self.c_feature) };
        if fid < 0 {
            None
        } else {
            Some(fid as u64)
        }
    }

    /// Get the value of a named field. If the field exists, it returns a [`FieldValue`] wrapper,
    /// that you need to unpack to a base type (string, float, etc).
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    /// If the field has an unsupported type, returns a [`GdalError::UnhandledFieldType`].
    ///
    /// If the field is null, returns `None`.
    pub fn field<S: AsRef<str>>(&self, name: S) -> Result<Option<FieldValue>> {
        let c_name = CString::new(name.as_ref())?;
        self._field(c_name)
    }

    fn _field(&self, c_name: CString) -> Result<Option<FieldValue>> {
        let field_id = unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_name.as_ptr()) };
        if field_id == -1 {
            Err(GdalError::InvalidFieldName {
                field_name: c_name.into_string()?,
                method_name: "OGR_F_GetFieldIndex",
            })
        } else {
            self.field_from_id(field_id)
        }
    }

    /// Get the value of a named field. If the field exists, it returns a [`FieldValue`] wrapper,
    /// that you need to unpack to a base type (string, float, etc).
    ///
    /// If the field has an unhandled type, returns a [`GdalError::UnhandledFieldType`].
    ///
    /// If the field is null, returns `None`.
    fn field_from_id(&self, field_id: i32) -> Result<Option<FieldValue>> {
        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_id) } != 0 {
            return Ok(None);
        }

        let field_defn = unsafe { gdal_sys::OGR_F_GetFieldDefnRef(self.c_feature, field_id) };
        let field_type = unsafe { gdal_sys::OGR_Fld_GetType(field_defn) };
        match field_type {
            OGRFieldType::OFTString => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsString(self.c_feature, field_id) };
                Ok(Some(FieldValue::StringValue(_string(rv))))
            }
            OGRFieldType::OFTStringList => {
                let rv = unsafe {
                    let ptr = gdal_sys::OGR_F_GetFieldAsStringList(self.c_feature, field_id);
                    _string_array(ptr)
                };
                Ok(Some(FieldValue::StringListValue(rv)))
            }
            OGRFieldType::OFTReal => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.c_feature, field_id) };
                Ok(Some(FieldValue::RealValue(rv as f64)))
            }
            OGRFieldType::OFTRealList => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr =
                        gdal_sys::OGR_F_GetFieldAsDoubleList(self.c_feature, field_id, &mut len);
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::RealListValue(rv)))
            }
            OGRFieldType::OFTInteger => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.c_feature, field_id) };
                Ok(Some(FieldValue::IntegerValue(rv as i32)))
            }
            OGRFieldType::OFTIntegerList => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr =
                        gdal_sys::OGR_F_GetFieldAsIntegerList(self.c_feature, field_id, &mut len);
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::IntegerListValue(rv)))
            }
            OGRFieldType::OFTInteger64 => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsInteger64(self.c_feature, field_id) };
                Ok(Some(FieldValue::Integer64Value(rv)))
            }
            OGRFieldType::OFTInteger64List => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr =
                        gdal_sys::OGR_F_GetFieldAsInteger64List(self.c_feature, field_id, &mut len);
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::Integer64ListValue(rv)))
            }
            OGRFieldType::OFTDateTime => Ok(Some(FieldValue::DateTimeValue(
                self._field_as_datetime(field_id)?,
            ))),
            OGRFieldType::OFTDate => Ok(Some(FieldValue::DateValue(
                self._field_as_datetime(field_id)?.date(),
            ))),
            _ => Err(GdalError::UnhandledFieldType {
                field_type,
                method_name: "OGR_Fld_GetType",
            }),
        }
    }

    /// Get the index of the named field.
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    fn field_idx_from_name(&self, field_name: &str) -> Result<i32> {
        let c_str_field_name = CString::new(field_name)?;
        let field_id =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if field_id == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            });
        }

        Ok(field_id)
    }

    /// Get the value of the specified field as a [`i32`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0))` on other kinds of errors.
    ///
    pub fn field_as_integer(&self, field_idx: i32) -> Result<Option<i32>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx as usize,
                method_name: "field_as_integer",
            });
        }

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.c_feature, field_idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`i32`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0))` on other kinds of errors.
    ///
    pub fn field_as_integer_by_name(&self, field_name: &str) -> Result<Option<i32>> {
        let field_idx = self.field_idx_from_name(field_name)?;

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.c_feature, field_idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`i64`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0))` on other kinds of errors.
    ///
    pub fn field_as_integer64_by_name(&self, field_name: &str) -> Result<Option<i64>> {
        let field_idx = self.field_idx_from_name(field_name)?;

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger64(self.c_feature, field_idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`i64`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0))` on other kinds of errors.
    ///
    pub fn field_as_integer64(&self, field_idx: i32) -> Result<Option<i64>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx as usize,
                method_name: "field_as_integer64",
            });
        }

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger64(self.c_feature, field_idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`f64`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0.))` on other kinds of errors.
    ///
    pub fn field_as_double_by_name(&self, field_name: &str) -> Result<Option<f64>> {
        let field_idx = self.field_idx_from_name(field_name)?;

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.c_feature, field_idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`f64`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0.))` on other kinds of errors.
    ///
    pub fn field_as_double(&self, field_idx: i32) -> Result<Option<f64>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx as usize,
                method_name: "field_as_double",
            });
        }

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.c_feature, field_idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`String`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    /// Returns `Ok(None)` if the field is null.
    ///
    pub fn field_as_string_by_name(&self, field_name: &str) -> Result<Option<String>> {
        let field_idx = self.field_idx_from_name(field_name)?;

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = _string(unsafe { gdal_sys::OGR_F_GetFieldAsString(self.c_feature, field_idx) });

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`String`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    ///
    pub fn field_as_string(&self, field_idx: i32) -> Result<Option<String>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx as usize,
                method_name: "field_as_string",
            });
        }

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = _string(unsafe { gdal_sys::OGR_F_GetFieldAsString(self.c_feature, field_idx) });

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`DateTime<FixedOffset>`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    /// Returns `Ok(None)` if the field is null.
    ///
    pub fn field_as_datetime_by_name(
        &self,
        field_name: &str,
    ) -> Result<Option<DateTime<FixedOffset>>> {
        let field_idx = self.field_idx_from_name(field_name)?;

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = self._field_as_datetime(field_idx)?;

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`DateTime<FixedOffset>`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    ///
    pub fn field_as_datetime(&self, field_idx: i32) -> Result<Option<DateTime<FixedOffset>>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx as usize,
                method_name: "field_as_datetime",
            });
        }

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let value = self._field_as_datetime(field_idx)?;

        Ok(Some(value))
    }

    fn _field_as_datetime(&self, field_id: c_int) -> Result<DateTime<FixedOffset>> {
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
            return Err(GdalError::OgrError {
                err: OGRErr::OGRERR_FAILURE,
                method_name: "OGR_F_GetFieldAsDateTime",
            });
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
            Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "geometry_by_name",
            })
        } else {
            self.geometry_by_index(idx as usize)
        }
    }

    pub fn geometry_by_index(&self, idx: usize) -> Result<&Geometry> {
        if idx >= self.geometry.len() {
            return Err(GdalError::InvalidFieldIndex {
                index: idx,
                method_name: "geometry_by_name",
            });
        }
        if !self.geometry[idx].has_gdal_ptr() {
            let c_geom = unsafe { gdal_sys::OGR_F_GetGeomFieldRef(self.c_feature, idx as i32) };
            if c_geom.is_null() {
                return Err(_last_null_pointer_err("OGR_F_GetGeomFieldRef"));
            }
            unsafe { self.geometry[idx].set_c_geometry(c_geom) };
        }
        Ok(&self.geometry[idx])
    }

    pub fn create<L: LayerAccess>(&self, lyr: &L) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_L_CreateFeature(lyr.c_layer(), self.c_feature) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_L_CreateFeature",
            });
        }
        Ok(())
    }

    pub fn set_field_string(&self, field_name: &str, value: &str) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let c_str_value = CString::new(value)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            });
        }
        unsafe { gdal_sys::OGR_F_SetFieldString(self.c_feature, idx, c_str_value.as_ptr()) };
        Ok(())
    }

    pub fn set_field_double(&self, field_name: &str, value: f64) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            });
        }
        unsafe { gdal_sys::OGR_F_SetFieldDouble(self.c_feature, idx, value as c_double) };
        Ok(())
    }

    pub fn set_field_integer(&self, field_name: &str, value: i32) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            });
        }
        unsafe { gdal_sys::OGR_F_SetFieldInteger(self.c_feature, idx, value as c_int) };
        Ok(())
    }

    pub fn set_field_integer64(&self, field_name: &str, value: i64) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            });
        }
        unsafe { gdal_sys::OGR_F_SetFieldInteger64(self.c_feature, idx, value as c_longlong) };
        Ok(())
    }

    pub fn set_field_datetime(&self, field_name: &str, value: DateTime<FixedOffset>) -> Result<()> {
        let c_str_field_name = CString::new(field_name)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature, c_str_field_name.as_ptr()) };
        if idx == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            });
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
        match value {
            FieldValue::RealValue(value) => self.set_field_double(field_name, *value),
            FieldValue::StringValue(ref value) => self.set_field_string(field_name, value.as_str()),
            FieldValue::IntegerValue(value) => self.set_field_integer(field_name, *value),
            FieldValue::Integer64Value(value) => self.set_field_integer64(field_name, *value),

            FieldValue::DateTimeValue(value) => self.set_field_datetime(field_name, *value),

            FieldValue::DateValue(value) => {
                self.set_field_datetime(field_name, value.and_hms(0, 0, 0))
            }
            _ => Err(GdalError::UnhandledFieldType {
                field_type: value.ogr_field_type(),
                method_name: "OGR_Fld_GetType",
            }),
        }
    }

    pub fn set_geometry(&mut self, geom: Geometry) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_F_SetGeometry(self.c_feature, geom.c_geometry()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_F_SetGeometry",
            });
        }
        self.geometry[0] = geom;
        Ok(())
    }
    pub fn field_count(&self) -> i32 {
        unsafe { gdal_sys::OGR_F_GetFieldCount(self.c_feature) }
    }

    pub fn fields(&self) -> FieldValueIterator {
        FieldValueIterator::with_feature(self)
    }
}

pub struct FieldValueIterator<'a> {
    feature: &'a Feature<'a>,
    idx: i32,
    count: i32,
}

impl<'a> FieldValueIterator<'a> {
    pub fn with_feature(feature: &'a Feature<'a>) -> Self {
        FieldValueIterator {
            feature,
            idx: 0,
            count: feature.field_count(),
        }
    }
}

impl<'a> Iterator for FieldValueIterator<'a> {
    type Item = (String, Option<FieldValue>);

    #[inline]
    fn next(&mut self) -> Option<(String, Option<FieldValue>)> {
        let idx = self.idx;
        if idx < self.count {
            self.idx += 1;
            let field_defn =
                unsafe { gdal_sys::OGR_F_GetFieldDefnRef(self.feature.c_feature, idx) };
            let field_name = unsafe { gdal_sys::OGR_Fld_GetNameRef(field_defn) };
            let name = _string(field_name);
            let fv: Option<(String, Option<FieldValue>)> = self
                .feature
                .field_from_id(idx)
                .ok()
                .map(|field_value| (name, field_value));
            //skip unknown types
            if fv.is_none() {
                return self.next();
            }
            fv
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match Some(self.count).map(|s| s.try_into().ok()).flatten() {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

impl<'a> Drop for Feature<'a> {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::OGR_F_Destroy(self.c_feature);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    IntegerValue(i32),
    IntegerListValue(Vec<i32>),
    Integer64Value(i64),
    Integer64ListValue(Vec<i64>),
    StringValue(String),
    StringListValue(Vec<String>),
    RealValue(f64),
    RealListValue(Vec<f64>),
    DateValue(Date<FixedOffset>),
    DateTimeValue(DateTime<FixedOffset>),
}

impl FieldValue {
    /// Interpret the value as `String`. Returns `None` if the value is something else.
    pub fn into_string(self) -> Option<String> {
        match self {
            FieldValue::StringValue(rv) => Some(rv),
            _ => None,
        }
    }

    /// Interpret the value as `f64`. Returns `None` if the value is something else.
    pub fn into_real(self) -> Option<f64> {
        match self {
            FieldValue::RealValue(rv) => Some(rv),
            _ => None,
        }
    }

    /// Interpret the value as `i32`. Returns `None` if the value is something else.
    pub fn into_int(self) -> Option<i32> {
        match self {
            FieldValue::IntegerValue(rv) => Some(rv),
            FieldValue::Integer64Value(rv) => rv.try_into().ok(),
            _ => None,
        }
    }

    /// Interpret the value as `i64`. Returns `None` if the value is something else.
    pub fn into_int64(self) -> Option<i64> {
        match self {
            FieldValue::IntegerValue(rv) => Some(rv as i64),
            FieldValue::Integer64Value(rv) => Some(rv),
            _ => None,
        }
    }

    /// Interpret the value as `Date`. Returns `None` if the value is something else.
    pub fn into_date(self) -> Option<Date<FixedOffset>> {
        match self {
            FieldValue::DateValue(rv) => Some(rv),
            FieldValue::DateTimeValue(rv) => Some(rv.date()),
            _ => None,
        }
    }

    /// Interpret the value as `DateTime`. Returns `None` if the value is something else.
    pub fn into_datetime(self) -> Option<DateTime<FixedOffset>> {
        match self {
            FieldValue::DateTimeValue(rv) => Some(rv),
            _ => None,
        }
    }

    pub fn ogr_field_type(&self) -> OGRFieldType::Type {
        match self {
            FieldValue::IntegerValue(_) => OGRFieldType::OFTInteger,
            FieldValue::IntegerListValue(_) => OGRFieldType::OFTIntegerList,
            FieldValue::Integer64Value(_) => OGRFieldType::OFTInteger64,
            FieldValue::Integer64ListValue(_) => OGRFieldType::OFTInteger64List,
            FieldValue::StringValue(_) => OGRFieldType::OFTString,
            FieldValue::StringListValue(_) => OGRFieldType::OFTStringList,
            FieldValue::RealValue(_) => OGRFieldType::OFTReal,
            FieldValue::RealListValue(_) => OGRFieldType::OFTRealList,
            FieldValue::DateValue(_) => OGRFieldType::OFTDate,
            FieldValue::DateTimeValue(_) => OGRFieldType::OFTDateTime,
        }
    }
}
