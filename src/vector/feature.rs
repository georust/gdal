use crate::utils::{_last_null_pointer_err, _string, _string_array};
use crate::vector::geometry::Geometry;
use crate::vector::OwnedLayer;
use crate::vector::{Defn, GeometryRef, LayerAccess};
use gdal_sys::{self, OGRErr, OGRFieldType, OGRLayerH};
use libc::{c_char, c_double, c_int, c_longlong};
use std::convert::TryInto;
use std::ffi::{CString, NulError};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr;

use chrono::{DateTime, Datelike, FixedOffset, LocalResult, NaiveDate, TimeZone, Timelike};

use crate::errors::*;
use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};
use std::slice;

foreign_type! {
    /// OGR Feature
    pub unsafe type Feature<'a> {
        type CType = libc::c_void;
        type PhantomData = &'a ();
        fn drop = gdal_sys::OGR_F_Destroy;
    }
}

impl<'a> Feature<'a> {
    pub fn new(defn: &'a Defn) -> Result<Feature> {
        let c_feature = unsafe { gdal_sys::OGR_F_Create(defn.as_ptr()) };
        if c_feature.is_null() {
            return Err(_last_null_pointer_err("OGR_F_Create"));
        };
        Ok(unsafe { Feature::from_ptr(c_feature) })
    }

    /// Returns the feature identifier, or `None` if none has been assigned.
    pub fn fid(&self) -> Option<u64> {
        let fid = unsafe { gdal_sys::OGR_F_GetFID(self.as_ptr()) };
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
        let idx = self.field_idx_from_name(name)?;
        self.field_from_id(idx)
    }

    /// Get the value of a named field. If the field exists, it returns a [`FieldValue`] wrapper,
    /// that you need to unpack to a base type (string, float, etc).
    ///
    /// If the field has an unhandled type, returns a [`GdalError::UnhandledFieldType`].
    ///
    /// If the field is null, returns `None`.
    fn field_from_id(&self, field_id: i32) -> Result<Option<FieldValue>> {
        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_id) } != 0 {
            return Ok(None);
        }

        let field_defn = unsafe { gdal_sys::OGR_F_GetFieldDefnRef(self.as_ptr(), field_id) };
        let field_type = unsafe { gdal_sys::OGR_Fld_GetType(field_defn) };
        match field_type {
            OGRFieldType::OFTString => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsString(self.as_ptr(), field_id) };
                Ok(Some(FieldValue::StringValue(_string(rv))))
            }
            OGRFieldType::OFTStringList => {
                let rv = unsafe {
                    let ptr = gdal_sys::OGR_F_GetFieldAsStringList(self.as_ptr(), field_id);
                    _string_array(ptr)
                };
                Ok(Some(FieldValue::StringListValue(rv)))
            }
            OGRFieldType::OFTReal => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.as_ptr(), field_id) };
                Ok(Some(FieldValue::RealValue(rv)))
            }
            OGRFieldType::OFTRealList => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr =
                        gdal_sys::OGR_F_GetFieldAsDoubleList(self.as_ptr(), field_id, &mut len);
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::RealListValue(rv)))
            }
            OGRFieldType::OFTInteger => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.as_ptr(), field_id) };
                Ok(Some(FieldValue::IntegerValue(rv)))
            }
            OGRFieldType::OFTIntegerList => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr =
                        gdal_sys::OGR_F_GetFieldAsIntegerList(self.as_ptr(), field_id, &mut len);
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::IntegerListValue(rv)))
            }
            OGRFieldType::OFTInteger64 => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsInteger64(self.as_ptr(), field_id) };
                Ok(Some(FieldValue::Integer64Value(rv)))
            }
            OGRFieldType::OFTInteger64List => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr =
                        gdal_sys::OGR_F_GetFieldAsInteger64List(self.as_ptr(), field_id, &mut len);
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::Integer64ListValue(rv)))
            }
            OGRFieldType::OFTDateTime => Ok(Some(FieldValue::DateTimeValue(
                self._field_as_datetime(field_id)?,
            ))),
            OGRFieldType::OFTDate => Ok(Some(FieldValue::DateValue(
                self._field_as_datetime(field_id)?.date_naive(),
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
    fn field_idx_from_name<S: AsRef<str>>(&self, field_name: S) -> Result<i32> {
        let c_str_field_name = CString::new(field_name.as_ref())?;
        let field_id =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.as_ptr(), c_str_field_name.as_ptr()) };
        if field_id == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.as_ref().to_string(),
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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.as_ptr(), field_idx) };

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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.as_ptr(), field_idx) };

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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger64(self.as_ptr(), field_idx) };

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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger64(self.as_ptr(), field_idx) };

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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.as_ptr(), field_idx) };

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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.as_ptr(), field_idx) };

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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
            return Ok(None);
        }

        let value = _string(unsafe { gdal_sys::OGR_F_GetFieldAsString(self.as_ptr(), field_idx) });

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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
            return Ok(None);
        }

        let value = _string(unsafe { gdal_sys::OGR_F_GetFieldAsString(self.as_ptr(), field_idx) });

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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
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

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.as_ptr(), field_idx) } != 0 {
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
                self.as_ptr(),
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
            (tzflag - 100) * 15 * 60
        };
        let rv = FixedOffset::east_opt(tzoffset_secs)
            .ok_or_else(|| GdalError::DateError(tzoffset_secs.to_string()))?
            .with_ymd_and_hms(
                year,
                month as u32,
                day as u32,
                hour as u32,
                minute as u32,
                second as u32,
            );
        match rv {
            LocalResult::None => Err(
                GdalError::DateError(format!("Unable to reconstruct valid date from fields: {year}, {month}, {day}, {hour}, {minute}, {second}"))
            ),
            LocalResult::Single(d) => Ok(d),
            LocalResult::Ambiguous(d1, d2) => Err(
                GdalError::DateError(format!("ambiguous date conversion; either '{d1}' or '{d2}'"))
            )
        }
    }

    /// Get the feature's geometry.
    pub fn geometry(&self) -> Option<&GeometryRef> {
        if self.geom_field_count() <= 0 {
            return None;
        }
        let c_geom = unsafe { gdal_sys::OGR_F_GetGeometryRef(self.as_ptr()) };
        Some(unsafe { GeometryRef::from_ptr(c_geom) })
    }

    pub fn geometry_by_name(&self, field_name: &str) -> Result<&GeometryRef> {
        let c_str_field_name = CString::new(field_name)?;
        let idx =
            unsafe { gdal_sys::OGR_F_GetGeomFieldIndex(self.as_ptr(), c_str_field_name.as_ptr()) };
        if idx == -1 {
            Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "geometry_by_name",
            })
        } else {
            self.geometry_by_index(idx as usize)
        }
    }

    pub fn geometry_by_index(&self, idx: usize) -> Result<&GeometryRef> {
        if idx as i32 >= self.geom_field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: idx,
                method_name: "geometry_by_index",
            });
        }
        let c_geom = unsafe { gdal_sys::OGR_F_GetGeomFieldRef(self.as_ptr(), idx as i32) };
        if c_geom.is_null() {
            return Err(_last_null_pointer_err("OGR_F_GetGeomFieldRef"));
        }
        Ok(unsafe { GeometryRef::from_ptr(c_geom) })
    }

    pub fn create<L: LayerAccess>(&self, lyr: &L) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_L_CreateFeature(lyr.c_layer(), self.as_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_L_CreateFeature",
            });
        }
        Ok(())
    }

    pub fn set_field_string(&self, field_name: &str, value: &str) -> Result<()> {
        let c_str_value = CString::new(value)?;
        let idx = self.field_idx_from_name(field_name)?;
        unsafe { gdal_sys::OGR_F_SetFieldString(self.as_ptr(), idx, c_str_value.as_ptr()) };
        Ok(())
    }

    pub fn set_field_string_list(&self, field_name: &str, value: &[&str]) -> Result<()> {
        let c_strings = value
            .iter()
            .map(|&value| CString::new(value))
            .collect::<std::result::Result<Vec<CString>, NulError>>()?;
        let c_str_ptrs = c_strings
            .iter()
            .map(|s| s.as_ptr())
            .chain(std::iter::once(ptr::null()))
            .collect::<Vec<*const c_char>>();
        // OGR_F_SetFieldStringList takes a CSLConstList, which is defined as *mut *mut c_char in
        // gdal-sys despite being constant.
        let c_value = c_str_ptrs.as_ptr() as *mut *mut c_char;
        let idx = self.field_idx_from_name(field_name)?;
        unsafe { gdal_sys::OGR_F_SetFieldStringList(self.as_ptr(), idx, c_value) };
        Ok(())
    }

    pub fn set_field_double(&self, field_name: &str, value: f64) -> Result<()> {
        let idx = self.field_idx_from_name(field_name)?;
        unsafe { gdal_sys::OGR_F_SetFieldDouble(self.as_ptr(), idx, value as c_double) };
        Ok(())
    }

    pub fn set_field_double_list(&self, field_name: &str, value: &[f64]) -> Result<()> {
        let idx = self.field_idx_from_name(field_name)?;
        unsafe {
            gdal_sys::OGR_F_SetFieldDoubleList(
                self.as_ptr(),
                idx,
                value.len() as c_int,
                value.as_ptr(),
            )
        };
        Ok(())
    }

    pub fn set_field_integer(&self, field_name: &str, value: i32) -> Result<()> {
        let idx = self.field_idx_from_name(field_name)?;
        unsafe { gdal_sys::OGR_F_SetFieldInteger(self.as_ptr(), idx, value as c_int) };
        Ok(())
    }

    pub fn set_field_integer_list(&self, field_name: &str, value: &[i32]) -> Result<()> {
        let idx = self.field_idx_from_name(field_name)?;
        unsafe {
            gdal_sys::OGR_F_SetFieldIntegerList(
                self.as_ptr(),
                idx,
                value.len() as c_int,
                value.as_ptr(),
            )
        };
        Ok(())
    }

    pub fn set_field_integer64(&self, field_name: &str, value: i64) -> Result<()> {
        let idx = self.field_idx_from_name(field_name)?;
        unsafe { gdal_sys::OGR_F_SetFieldInteger64(self.as_ptr(), idx, value as c_longlong) };
        Ok(())
    }

    pub fn set_field_integer64_list(&self, field_name: &str, value: &[i64]) -> Result<()> {
        let idx = self.field_idx_from_name(field_name)?;
        unsafe {
            gdal_sys::OGR_F_SetFieldInteger64List(
                self.as_ptr(),
                idx,
                value.len() as c_int,
                value.as_ptr(),
            )
        };
        Ok(())
    }

    pub fn set_field_datetime(&self, field_name: &str, value: DateTime<FixedOffset>) -> Result<()> {
        let idx = self.field_idx_from_name(field_name)?;

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
                self.as_ptr(),
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
            FieldValue::IntegerValue(value) => self.set_field_integer(field_name, *value),
            FieldValue::IntegerListValue(value) => self.set_field_integer_list(field_name, value),
            FieldValue::Integer64Value(value) => self.set_field_integer64(field_name, *value),
            FieldValue::Integer64ListValue(value) => {
                self.set_field_integer64_list(field_name, value)
            }
            FieldValue::StringValue(ref value) => self.set_field_string(field_name, value.as_str()),
            FieldValue::StringListValue(ref value) => {
                let strs = value.iter().map(String::as_str).collect::<Vec<&str>>();
                self.set_field_string_list(field_name, &strs)
            }
            FieldValue::RealValue(value) => self.set_field_double(field_name, *value),
            FieldValue::RealListValue(value) => self.set_field_double_list(field_name, value),
            FieldValue::DateValue(value) => {
                let dv = value
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| GdalError::DateError("offset to midnight".into()))?;
                let dt = DateTime::from_naive_utc_and_offset(
                    dv,
                    FixedOffset::east_opt(0)
                        .ok_or_else(|| GdalError::DateError("utc offset".into()))?,
                );
                self.set_field_datetime(field_name, dt)
            }
            FieldValue::DateTimeValue(value) => self.set_field_datetime(field_name, *value),
        }
    }

    pub fn set_geometry(&mut self, geom: Geometry) -> Result<()> {
        let rv = unsafe { gdal_sys::OGR_F_SetGeometry(self.as_ptr(), geom.as_ptr()) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "OGR_F_SetGeometry",
            });
        }
        Ok(())
    }
    pub fn field_count(&self) -> i32 {
        unsafe { gdal_sys::OGR_F_GetFieldCount(self.as_ptr()) }
    }

    pub fn geom_field_count(&self) -> i32 {
        unsafe { gdal_sys::OGR_F_GetGeomFieldCount(self.as_ptr()) }
    }

    pub fn fields(&self) -> FieldValueIterator {
        FieldValueIterator::with_feature(self)
    }
}

impl Debug for Feature<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let fields = self.fields().collect::<Vec<_>>();
        f.debug_struct("Feature")
            .field("fid", &self.fid())
            .field("geometry", &self.geometry())
            .field("fields", &fields)
            .finish()
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
            let field_defn = unsafe { gdal_sys::OGR_F_GetFieldDefnRef(self.feature.as_ptr(), idx) };
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
        match Some(self.count).and_then(|s| s.try_into().ok()) {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

pub struct FeatureIterator<'a> {
    c_layer: OGRLayerH,
    size_hint: Option<usize>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Iterator for FeatureIterator<'a> {
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetNextFeature(self.c_layer) };
        if c_feature.is_null() {
            None
        } else {
            Some(unsafe { Feature::from_ptr(c_feature) })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.size_hint {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

impl<'a> FeatureIterator<'a> {
    pub(crate) fn _with_layer<L: LayerAccess>(layer: &'a L) -> Self {
        let size_hint = layer.try_feature_count().and_then(|s| s.try_into().ok());
        Self {
            c_layer: unsafe { layer.c_layer() },
            size_hint,
            _lifetime: PhantomData,
        }
    }
}

pub struct OwnedFeatureIterator {
    pub(crate) layer: OwnedLayer,
    size_hint: Option<usize>,
}

impl<'a> Iterator for &'a mut OwnedFeatureIterator
where
    Self: 'a,
{
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetNextFeature(self.layer.c_layer()) };

        if c_feature.is_null() {
            return None;
        }

        Some(unsafe { Feature::from_ptr(c_feature) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.size_hint {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

impl OwnedFeatureIterator {
    pub(crate) fn _with_layer(layer: OwnedLayer) -> Self {
        let size_hint = layer.try_feature_count().and_then(|s| s.try_into().ok());
        Self { layer, size_hint }
    }

    pub fn into_layer(self) -> OwnedLayer {
        self.layer
    }
}

impl AsMut<OwnedFeatureIterator> for OwnedFeatureIterator {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl From<OwnedFeatureIterator> for OwnedLayer {
    fn from(feature_iterator: OwnedFeatureIterator) -> Self {
        feature_iterator.into_layer()
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
    DateValue(NaiveDate),
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

    /// Interpret the value as `NaiveDate`. Returns `None` if the value is something else.
    pub fn into_date(self) -> Option<NaiveDate> {
        match self {
            FieldValue::DateValue(rv) => Some(rv),
            FieldValue::DateTimeValue(rv) => Some(rv.date_naive()),
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

pub fn field_type_to_name(ty: OGRFieldType::Type) -> String {
    let rv = unsafe { gdal_sys::OGR_GetFieldTypeName(ty) };
    _string(rv)
}

#[test]
pub fn test_field_type_to_name() {
    assert_eq!(field_type_to_name(OGRFieldType::OFTReal), "Real");
    // We don't care what it returns when passed an invalid value, just that it doesn't crash.
    field_type_to_name(4372521);
}
