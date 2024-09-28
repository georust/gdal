use crate::spatial_ref::SpatialRef;
use crate::utils::{_last_null_pointer_err, _string};
use crate::vector::LayerAccess;
use gdal_sys::{
    self, OGRFeatureDefnH, OGRFieldDefnH, OGRFieldType, OGRGeomFieldDefnH, OGRwkbGeometryType,
};
use libc::c_int;

use crate::errors::*;

/// Layer definition
///
/// Defines the fields available for features in a layer.
#[derive(Debug)]
pub struct Defn {
    c_defn: OGRFeatureDefnH,
}

impl Defn {
    /// Creates a new Defn by wrapping a C pointer
    ///
    /// # Safety
    /// This method operates on a raw C pointer
    pub unsafe fn from_c_defn(c_defn: OGRFeatureDefnH) -> Defn {
        Defn { c_defn }
    }

    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    pub unsafe fn c_defn(&self) -> OGRFeatureDefnH {
        self.c_defn
    }

    /// Iterate over the field schema of this layer.
    pub fn fields(&self) -> FieldIterator {
        let total = unsafe { gdal_sys::OGR_FD_GetFieldCount(self.c_defn) } as isize;
        FieldIterator {
            defn: self,
            c_feature_defn: self.c_defn,
            next_id: 0,
            total,
        }
    }

    /// Iterate over the geometry field schema of this layer.
    pub fn geom_fields(&self) -> GeomFieldIterator {
        let total = unsafe { gdal_sys::OGR_FD_GetGeomFieldCount(self.c_defn) } as isize;
        GeomFieldIterator {
            defn: self,
            c_feature_defn: self.c_defn,
            next_id: 0,
            total,
        }
    }

    pub fn from_layer<L: LayerAccess>(lyr: &L) -> Defn {
        let c_defn = unsafe { gdal_sys::OGR_L_GetLayerDefn(lyr.c_layer()) };
        Defn { c_defn }
    }

    /// Get the geometry type of the first geometry field
    pub fn geometry_type(&self) -> OGRwkbGeometryType::Type {
        unsafe { gdal_sys::OGR_FD_GetGeomType(self.c_defn) }
    }
}

pub struct FieldIterator<'a> {
    defn: &'a Defn,
    c_feature_defn: OGRFeatureDefnH,
    next_id: isize,
    total: isize,
}

impl<'a> Iterator for FieldIterator<'a> {
    type Item = Field<'a>;

    #[inline]
    fn next(&mut self) -> Option<Field<'a>> {
        if self.next_id == self.total {
            return None;
        }
        let field = Field {
            _defn: self.defn,
            c_field_defn: unsafe {
                gdal_sys::OGR_FD_GetFieldDefn(self.c_feature_defn, self.next_id as c_int)
            },
        };
        self.next_id += 1;
        Some(field)
    }
}

pub struct Field<'a> {
    _defn: &'a Defn,
    c_field_defn: OGRFieldDefnH,
}

impl<'a> Field<'a> {
    /// Get the name of this field.
    pub fn name(&'a self) -> String {
        let rv = unsafe { gdal_sys::OGR_Fld_GetNameRef(self.c_field_defn) };
        _string(rv)
    }

    /// Get the alternative name (alias) of this field.
    ///
    /// This API is new as of GDAL 3.2.
    #[cfg(any(major_ge_4, all(major_is_3, minor_ge_2)))]
    pub fn alternative_name(&'a self) -> String {
        let rv = unsafe { gdal_sys::OGR_Fld_GetAlternativeNameRef(self.c_field_defn) };
        _string(rv)
    }

    /// Get the data type of this field.
    pub fn field_type(&'a self) -> OGRFieldType::Type {
        unsafe { gdal_sys::OGR_Fld_GetType(self.c_field_defn) }
    }

    /// Get the formatting width for this field.
    ///
    /// Zero means no specified width.
    pub fn width(&'a self) -> i32 {
        unsafe { gdal_sys::OGR_Fld_GetWidth(self.c_field_defn) }
    }

    /// Get the formatting precision for this field.
    ///
    /// This should normally be zero for fields of types other than Real.
    pub fn precision(&'a self) -> i32 {
        unsafe { gdal_sys::OGR_Fld_GetPrecision(self.c_field_defn) }
    }

    /// Return whether this field can receive null values.
    pub fn is_nullable(&'a self) -> bool {
        unsafe { gdal_sys::OGR_Fld_IsNullable(self.c_field_defn) != 0 }
    }

    /// Return whether this field has a unique constraint.
    ///
    /// This API is new as of GDAL 3.2.
    #[cfg(any(major_ge_4, all(major_is_3, minor_ge_2)))]
    pub fn is_unique(&'a self) -> bool {
        unsafe { gdal_sys::OGR_Fld_IsUnique(self.c_field_defn) != 0 }
    }

    /// Get default field value.
    pub fn default_value(&'a self) -> Option<String> {
        let rv = unsafe { gdal_sys::OGR_Fld_GetDefault(self.c_field_defn) };
        if rv.is_null() {
            return None;
        }
        Some(_string(rv))
    }
}

pub struct GeomFieldIterator<'a> {
    defn: &'a Defn,
    c_feature_defn: OGRFeatureDefnH,
    next_id: isize,
    total: isize,
}

impl<'a> Iterator for GeomFieldIterator<'a> {
    type Item = GeomField<'a>;

    #[inline]
    fn next(&mut self) -> Option<GeomField<'a>> {
        if self.next_id == self.total {
            return None;
        }
        let field = GeomField {
            _defn: self.defn,
            c_field_defn: unsafe {
                gdal_sys::OGR_FD_GetGeomFieldDefn(self.c_feature_defn, self.next_id as c_int)
            },
        };
        self.next_id += 1;
        Some(field)
    }
}

// http://gdal.org/classOGRGeomFieldDefn.html
pub struct GeomField<'a> {
    _defn: &'a Defn,
    c_field_defn: OGRGeomFieldDefnH,
}

impl<'a> GeomField<'a> {
    /// Get the name of this field.
    pub fn name(&'a self) -> String {
        let rv = unsafe { gdal_sys::OGR_GFld_GetNameRef(self.c_field_defn) };
        _string(rv)
    }

    pub fn field_type(&'a self) -> OGRwkbGeometryType::Type {
        unsafe { gdal_sys::OGR_GFld_GetType(self.c_field_defn) }
    }

    pub fn spatial_ref(&'a self) -> Result<SpatialRef> {
        let c_obj = unsafe { gdal_sys::OGR_GFld_GetSpatialRef(self.c_field_defn) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OGR_GFld_GetSpatialRef"));
        }
        unsafe { SpatialRef::from_c_obj(c_obj) }
    }
}
