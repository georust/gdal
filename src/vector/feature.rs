use std::{
    convert::TryInto,
    ffi::{c_char, c_double, c_int, c_longlong, CString, NulError},
    ptr, slice,
};

use chrono::{DateTime, Datelike, FixedOffset, LocalResult, NaiveDate, TimeZone, Timelike};
use gdal_sys::{OGRErr, OGRFeatureH, OGRFieldType, OGRLayerH};

use crate::utils::{_last_null_pointer_err, _string, _string_array};
use crate::vector::geometry::Geometry;
use crate::vector::{Defn, LayerAccess, OwnedLayer};

use crate::errors::*;

/// OGR Feature
#[derive(Debug)]
pub struct Feature<'a> {
    _defn: &'a Defn,
    c_feature: OGRFeatureH,
    geometry: Vec<Geometry>,
}

impl<'a> Feature<'a> {
    pub fn new(defn: &'a Defn) -> Result<Feature<'a>> {
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
    pub unsafe fn from_c_feature(defn: &'a Defn, c_feature: OGRFeatureH) -> Feature<'a> {
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

    /// Get the value of a field. If the field exists, it returns a [`FieldValue`] wrapper,
    /// that you need to unpack to a base type (string, float, etc).
    ///
    /// If the field has an unhandled type, returns a [`GdalError::UnhandledFieldType`].
    ///
    /// If the field is null, returns `None`.
    pub fn field(&self, field_idx: usize) -> Result<Option<FieldValue>> {
        let field_idx = field_idx.try_into()?;

        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, field_idx) } != 0 {
            return Ok(None);
        }

        let field_defn = unsafe { gdal_sys::OGR_F_GetFieldDefnRef(self.c_feature, field_idx) };
        let field_type = unsafe { gdal_sys::OGR_Fld_GetType(field_defn) };
        match field_type {
            OGRFieldType::OFTString => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsString(self.c_feature, field_idx) };
                Ok(_string(rv).map(FieldValue::StringValue))
            }
            OGRFieldType::OFTStringList => {
                let rv = unsafe {
                    let ptr = gdal_sys::OGR_F_GetFieldAsStringList(self.c_feature, field_idx);
                    _string_array(ptr)
                };
                Ok(Some(FieldValue::StringListValue(rv)))
            }
            OGRFieldType::OFTReal => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.c_feature, field_idx) };
                Ok(Some(FieldValue::RealValue(rv)))
            }
            OGRFieldType::OFTRealList => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr =
                        gdal_sys::OGR_F_GetFieldAsDoubleList(self.c_feature, field_idx, &mut len);
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::RealListValue(rv)))
            }
            OGRFieldType::OFTInteger => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.c_feature, field_idx) };
                Ok(Some(FieldValue::IntegerValue(rv)))
            }
            OGRFieldType::OFTIntegerList => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr =
                        gdal_sys::OGR_F_GetFieldAsIntegerList(self.c_feature, field_idx, &mut len);
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::IntegerListValue(rv)))
            }
            OGRFieldType::OFTInteger64 => {
                let rv = unsafe { gdal_sys::OGR_F_GetFieldAsInteger64(self.c_feature, field_idx) };
                Ok(Some(FieldValue::Integer64Value(rv)))
            }
            OGRFieldType::OFTInteger64List => {
                let rv = unsafe {
                    let mut len: i32 = 0;
                    let ptr = gdal_sys::OGR_F_GetFieldAsInteger64List(
                        self.c_feature,
                        field_idx,
                        &mut len,
                    );
                    slice::from_raw_parts(ptr, len as usize).to_vec()
                };
                Ok(Some(FieldValue::Integer64ListValue(rv)))
            }
            OGRFieldType::OFTDateTime => Ok(Some(FieldValue::DateTimeValue(
                self._field_as_datetime(field_idx)?,
            ))),
            OGRFieldType::OFTDate => Ok(Some(FieldValue::DateValue(
                self._field_as_datetime(field_idx)?.date_naive(),
            ))),
            _ => Err(GdalError::UnhandledFieldType {
                field_type,
                method_name: "OGR_Fld_GetType",
            }),
        }
    }

    /// Get the index of a field.
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    /// Calling [`Defn::field_index`] once and caching the index can be faster, and should be preferred.
    pub fn field_index<S: AsRef<str>>(&self, field_name: S) -> Result<usize> {
        self._field_index(field_name.as_ref())
    }

    fn _field_index(&self, field_name: &str) -> Result<usize> {
        let c_str_field_name = CString::new(field_name)?;
        let field_idx =
            unsafe { gdal_sys::OGR_F_GetFieldIndex(self.c_feature(), c_str_field_name.as_ptr()) };
        if field_idx == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetFieldIndex",
            });
        }

        let field_index = field_idx.try_into()?;
        Ok(field_index)
    }

    /// Get the index of a geometry field.
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldName`].
    ///
    /// Calling [`Defn::geometry_field_index`] once and caching the index can be faster, and should be preferred.
    pub fn geometry_field_index<S: AsRef<str>>(&self, field_name: S) -> Result<usize> {
        self._geometry_field_index(field_name.as_ref())
    }

    fn _geometry_field_index(&self, field_name: &str) -> Result<usize> {
        let c_str_field_name = CString::new(field_name)?;
        let field_idx = unsafe {
            gdal_sys::OGR_F_GetGeomFieldIndex(self.c_feature(), c_str_field_name.as_ptr())
        };
        if field_idx == -1 {
            return Err(GdalError::InvalidFieldName {
                field_name: field_name.to_string(),
                method_name: "OGR_F_GetGeomFieldIndex",
            });
        }

        let field_index = field_idx.try_into()?;
        Ok(field_index)
    }

    /// Get the value of the specified field as a [`i32`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0))` on other kinds of errors.
    ///
    pub fn field_as_integer(&self, field_idx: usize) -> Result<Option<i32>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx,
                method_name: "field_as_integer",
            });
        }

        let idx = field_idx.try_into()?;
        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger(self.c_feature, idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`i64`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0))` on other kinds of errors.
    ///
    pub fn field_as_integer64(&self, field_idx: usize) -> Result<Option<i64>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx,
                method_name: "field_as_integer64",
            });
        }

        let idx = field_idx.try_into()?;
        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsInteger64(self.c_feature, idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`f64`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    /// Returns `Ok(Some(0.))` on other kinds of errors.
    ///
    pub fn field_as_double(&self, field_idx: usize) -> Result<Option<f64>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx,
                method_name: "field_as_double",
            });
        }

        let idx = field_idx.try_into()?;
        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, idx) } != 0 {
            return Ok(None);
        }

        let value = unsafe { gdal_sys::OGR_F_GetFieldAsDouble(self.c_feature, idx) };

        Ok(Some(value))
    }

    /// Get the value of the specified field as a [`String`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    ///
    pub fn field_as_string(&self, field_idx: usize) -> Result<Option<String>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx,
                method_name: "field_as_string",
            });
        }

        let idx = field_idx.try_into()?;
        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, idx) } != 0 {
            return Ok(None);
        }

        let value = _string(unsafe { gdal_sys::OGR_F_GetFieldAsString(self.c_feature, idx) });
        Ok(value)
    }

    /// Get the value of the specified field as a [`DateTime<FixedOffset>`].
    ///
    /// If the field is missing, returns [`GdalError::InvalidFieldIndex`].
    ///
    /// Returns `Ok(None)` if the field is null.
    ///
    pub fn field_as_datetime(&self, field_idx: usize) -> Result<Option<DateTime<FixedOffset>>> {
        if field_idx >= self.field_count() {
            return Err(GdalError::InvalidFieldIndex {
                index: field_idx,
                method_name: "field_as_datetime",
            });
        }

        let idx = field_idx.try_into()?;
        if unsafe { gdal_sys::OGR_F_IsFieldNull(self.c_feature, idx) } != 0 {
            return Ok(None);
        }

        let value = self._field_as_datetime(idx)?;

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
    pub fn geometry(&self) -> Option<&Geometry> {
        match self.geometry.first() {
            Some(geom) => {
                if !geom.has_gdal_ptr() {
                    let c_geom = unsafe { gdal_sys::OGR_F_GetGeometryRef(self.c_feature) };
                    unsafe { geom.set_c_geometry(c_geom) };
                }
                Some(geom)
            }
            None => None,
        }
    }

    pub fn geometry_by_index(&self, idx: usize) -> Result<&Geometry> {
        if idx >= self.geometry.len() {
            return Err(GdalError::InvalidFieldIndex {
                index: idx,
                method_name: "geometry_by_index",
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

    pub fn set_field_string(&mut self, field_idx: usize, value: &str) -> Result<()> {
        let c_str_value = CString::new(value)?;
        let idx = field_idx.try_into()?;
        unsafe { gdal_sys::OGR_F_SetFieldString(self.c_feature, idx, c_str_value.as_ptr()) };
        Ok(())
    }

    pub fn set_field_string_list(&mut self, field_idx: usize, value: &[&str]) -> Result<()> {
        let idx = field_idx.try_into()?;
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
        unsafe { gdal_sys::OGR_F_SetFieldStringList(self.c_feature, idx, c_value) };
        Ok(())
    }

    pub fn set_field_double(&mut self, field_idx: usize, value: f64) -> Result<()> {
        let idx = field_idx.try_into()?;
        unsafe { gdal_sys::OGR_F_SetFieldDouble(self.c_feature, idx, value as c_double) };
        Ok(())
    }

    pub fn set_field_double_list(&mut self, field_idx: usize, value: &[f64]) -> Result<()> {
        let idx = field_idx.try_into()?;
        unsafe {
            gdal_sys::OGR_F_SetFieldDoubleList(
                self.c_feature,
                idx,
                value.len() as c_int,
                value.as_ptr(),
            )
        };
        Ok(())
    }

    pub fn set_field_integer(&mut self, field_idx: usize, value: i32) -> Result<()> {
        let idx = field_idx.try_into()?;
        unsafe { gdal_sys::OGR_F_SetFieldInteger(self.c_feature, idx, value as c_int) };
        Ok(())
    }

    pub fn set_field_integer_list(&mut self, field_idx: usize, value: &[i32]) -> Result<()> {
        let idx = field_idx.try_into()?;
        unsafe {
            gdal_sys::OGR_F_SetFieldIntegerList(
                self.c_feature,
                idx,
                value.len() as c_int,
                value.as_ptr(),
            )
        };
        Ok(())
    }

    pub fn set_field_integer64(&mut self, field_idx: usize, value: i64) -> Result<()> {
        let idx = field_idx.try_into()?;
        unsafe { gdal_sys::OGR_F_SetFieldInteger64(self.c_feature, idx, value as c_longlong) };
        Ok(())
    }

    pub fn set_field_integer64_list(&mut self, field_idx: usize, value: &[i64]) -> Result<()> {
        let idx = field_idx.try_into()?;
        unsafe {
            gdal_sys::OGR_F_SetFieldInteger64List(
                self.c_feature,
                idx,
                value.len() as c_int,
                value.as_ptr(),
            )
        };
        Ok(())
    }

    pub fn set_field_datetime(
        &mut self,
        field_idx: usize,
        value: DateTime<FixedOffset>,
    ) -> Result<()> {
        let idx = field_idx.try_into()?;
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

    pub fn set_field(&mut self, field_idx: usize, value: &FieldValue) -> Result<()> {
        match value {
            FieldValue::IntegerValue(value) => self.set_field_integer(field_idx, *value),
            FieldValue::IntegerListValue(value) => self.set_field_integer_list(field_idx, value),
            FieldValue::Integer64Value(value) => self.set_field_integer64(field_idx, *value),
            FieldValue::Integer64ListValue(value) => {
                self.set_field_integer64_list(field_idx, value)
            }
            FieldValue::StringValue(ref value) => self.set_field_string(field_idx, value.as_str()),
            FieldValue::StringListValue(ref value) => {
                let strs = value.iter().map(String::as_str).collect::<Vec<&str>>();
                self.set_field_string_list(field_idx, &strs)
            }
            FieldValue::RealValue(value) => self.set_field_double(field_idx, *value),
            FieldValue::RealListValue(value) => self.set_field_double_list(field_idx, value),
            FieldValue::DateValue(value) => {
                let dv = value
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| GdalError::DateError("offset to midnight".into()))?;
                let dt = DateTime::from_naive_utc_and_offset(
                    dv,
                    FixedOffset::east_opt(0)
                        .ok_or_else(|| GdalError::DateError("utc offset".into()))?,
                );
                self.set_field_datetime(field_idx, dt)
            }
            FieldValue::DateTimeValue(value) => self.set_field_datetime(field_idx, *value),
        }
    }

    /// Clear a field, marking it as null.
    ///
    /// See: [`OGRFeature::SetFieldNull`][SetFieldNull]
    ///
    /// [SetFieldNull]: https://gdal.org/api/ogrfeature_cpp.html#_CPPv4N10OGRFeature12SetFieldNullEi
    pub fn set_field_null(&mut self, field_idx: usize) -> Result<()> {
        let idx = field_idx.try_into()?;
        unsafe { gdal_sys::OGR_F_SetFieldNull(self.c_feature(), idx) };
        Ok(())
    }

    /// Clear a field, marking it as unset.
    ///
    /// See: [`OGRFeature::UnsetField`][UnsetField]
    ///
    /// [UnsetField]: https://gdal.org/api/ogrfeature_cpp.html#_CPPv4N10OGRFeature10UnsetFieldEi
    pub fn unset_field(&mut self, field_idx: usize) -> Result<()> {
        let idx = field_idx.try_into()?;
        unsafe { gdal_sys::OGR_F_UnsetField(self.c_feature(), idx) };
        Ok(())
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

    pub fn field_count(&self) -> usize {
        let count = unsafe { gdal_sys::OGR_F_GetFieldCount(self.c_feature) };
        count as usize
    }

    pub fn fields(&self) -> FieldValueIterator<'_> {
        FieldValueIterator::with_feature(self)
    }
}

pub struct FieldValueIterator<'a> {
    feature: &'a Feature<'a>,
    idx: usize,
    count: usize,
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

impl Iterator for FieldValueIterator<'_> {
    type Item = (String, Option<FieldValue>);

    #[inline]
    fn next(&mut self) -> Option<(String, Option<FieldValue>)> {
        let idx = self.idx;
        if idx < self.count {
            self.idx += 1;
            let field_defn =
                unsafe { gdal_sys::OGR_F_GetFieldDefnRef(self.feature.c_feature, idx as c_int) };
            let field_name = unsafe { gdal_sys::OGR_Fld_GetNameRef(field_defn) };
            let name = _string(field_name).unwrap_or_default();
            let fv: Option<(String, Option<FieldValue>)> = self
                .feature
                .field(idx)
                .ok()
                .map(|field_value| (name, field_value));
            // skip unknown types
            if fv.is_none() {
                return self.next();
            }
            fv
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl Drop for Feature<'_> {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::OGR_F_Destroy(self.c_feature);
        }
    }
}

pub struct FeatureIterator<'a> {
    defn: &'a Defn,
    c_layer: OGRLayerH,
    size_hint: Option<usize>,
}

impl<'a> Iterator for FeatureIterator<'a> {
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetNextFeature(self.c_layer) };
        if c_feature.is_null() {
            None
        } else {
            Some(unsafe { Feature::from_c_feature(self.defn, c_feature) })
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
        let defn = layer.defn();
        let size_hint = layer.try_feature_count().and_then(|s| s.try_into().ok());
        Self {
            c_layer: unsafe { layer.c_layer() },
            size_hint,
            defn,
        }
    }
}

impl Drop for FeatureIterator<'_> {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::OGR_L_ResetReading(self.c_layer);
        }
    }
}

pub struct OwnedFeatureIterator {
    pub(crate) layer: Option<OwnedLayer>,
    size_hint: Option<usize>,
}

impl<'a> Iterator for &'a mut OwnedFeatureIterator
where
    Self: 'a,
{
    type Item = Feature<'a>;

    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { gdal_sys::OGR_L_GetNextFeature(self.layer().c_layer()) };

        if c_feature.is_null() {
            return None;
        }

        Some(unsafe {
            // We have to convince the compiler that our `Defn` adheres to our iterator lifetime `<'a>`
            let defn: &'a Defn = std::mem::transmute::<&'_ _, &'a _>(self.layer().defn());

            Feature::from_c_feature(defn, c_feature)
        })
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
        Self {
            layer: Some(layer),
            size_hint,
        }
    }

    pub fn into_layer(mut self) -> OwnedLayer {
        self.layer.take().expect("layer must be set")
    }

    fn layer(&self) -> &OwnedLayer {
        self.layer.as_ref().expect("layer must be set")
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

impl Drop for OwnedFeatureIterator {
    fn drop(&mut self) {
        if let Some(layer) = &self.layer {
            unsafe {
                gdal_sys::OGR_L_ResetReading(layer.c_layer());
            }
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
    _string(rv).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fixture;
    use crate::Dataset;

    #[test]
    fn test_field_type_to_name() {
        assert_eq!(field_type_to_name(OGRFieldType::OFTReal), "Real");
        // We don't care what it returns when passed an invalid value, just that it doesn't crash.
        field_type_to_name(4372521);
    }

    #[test]
    fn test_field_set_null() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();

        let mut layer = ds.layers().next().expect("layer");
        let mut feature = layer.features().next().expect("feature");
        let highway_idx = feature.field_index("highway").unwrap();
        feature.set_field_null(highway_idx).unwrap();
        assert!(feature.field(highway_idx).unwrap().is_none());
    }

    #[test]
    fn test_field_unset() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();

        let mut layer = ds.layers().next().expect("layer");
        let mut feature = layer.features().next().expect("feature");
        let highway_idx = feature.field_index("highway").unwrap();
        feature.unset_field(highway_idx).unwrap();
    }
}
