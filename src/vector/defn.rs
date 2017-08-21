use libc::{c_int, c_void};
use utils::{_last_null_pointer_err, _string};
use gdal_sys::ogr;
use vector::layer::Layer;
use vector::geometry::WkbType;
use spatial_ref::SpatialRef;
use gdal_major_object::MajorObject;
use gdal_sys::ogr_enums::OGRFieldType;

use errors::*;

/// Layer definition
///
/// Defines the fields available for features in a layer.
pub struct Defn {
    c_defn: *const c_void,
}

impl Defn {
    pub unsafe fn _with_c_defn(c_defn: *const c_void) -> Defn {
        Defn{c_defn: c_defn}
    }

    pub unsafe fn c_defn(&self) -> *const c_void { self.c_defn }

    /// Iterate over the field schema of this layer.
    pub fn fields(&self) -> FieldIterator {
        let total = unsafe { ogr::OGR_FD_GetFieldCount(self.c_defn) } as isize;
        return FieldIterator{
            defn: self,
            c_feature_defn: self.c_defn,
            next_id: 0,
            total: total
        };
    }

    /// Iterate over the geometry field schema of this layer.
    pub fn geom_fields(&self) -> GeomFieldIterator {
        let total = unsafe { ogr::OGR_FD_GetGeomFieldCount(self.c_defn) } as isize;
        return GeomFieldIterator{
            defn: self,
            c_feature_defn: self.c_defn,
            next_id: 0,
            total: total
        };
    }

    pub fn from_layer(lyr: &Layer) -> Defn {
        let c_defn = unsafe { ogr::OGR_L_GetLayerDefn(lyr.gdal_object_ptr())};
            Defn {c_defn: c_defn}
        }
}

pub struct FieldIterator<'a> {
    defn: &'a Defn,
    c_feature_defn: *const c_void,
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
        let field = Field{
            _defn: self.defn,
            c_field_defn: unsafe { ogr::OGR_FD_GetFieldDefn(
                self.c_feature_defn,
                self.next_id as c_int
            ) }
        };
        self.next_id += 1;
        return Some(field);
    }
}

pub struct Field<'a> {
    _defn: &'a Defn,
    c_field_defn: *const c_void,
}

impl<'a> Field<'a> {
    /// Get the name of this field.
    pub fn name(&'a self) -> String {
        let rv = unsafe { ogr::OGR_Fld_GetNameRef(self.c_field_defn) };
        return _string(rv);
    }

    pub fn field_type(&'a self) -> OGRFieldType {
        unsafe { ogr::OGR_Fld_GetType(self.c_field_defn) }
    }

    pub fn width(&'a self) -> i32 {
        unsafe { ogr::OGR_Fld_GetWidth(self.c_field_defn) }
    }

    pub fn precision(&'a self) -> i32 {
        unsafe { ogr::OGR_Fld_GetPrecision(self.c_field_defn) }
    }
}

pub struct GeomFieldIterator<'a> {
    defn: &'a Defn,
    c_feature_defn: *const c_void,
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
        let field = GeomField{
            _defn: self.defn,
            c_field_defn: unsafe { ogr::OGR_FD_GetGeomFieldDefn(
                self.c_feature_defn,
                self.next_id as c_int
            ) }
        };
        self.next_id += 1;
        return Some(field);
    }
}

// http://gdal.org/classOGRGeomFieldDefn.html
pub struct GeomField<'a> {
    _defn: &'a Defn,
    c_field_defn: *const c_void,
}

impl<'a> GeomField<'a> {
    /// Get the name of this field.
    pub fn name(&'a self) -> String {
        let rv = unsafe { ogr::OGR_GFld_GetNameRef(self.c_field_defn) };
        return _string(rv);
    }

    pub fn field_type(&'a self) -> WkbType {
        let ogr_type = unsafe { ogr::OGR_GFld_GetType(self.c_field_defn) };
        WkbType::from_ogr_type(ogr_type)
    }

    pub fn spatial_ref(&'a self) -> Result<SpatialRef> {
        let c_obj = unsafe { ogr::OGR_GFld_GetSpatialRef(self.c_field_defn) };
        if c_obj.is_null() {
            return Err(_last_null_pointer_err("OGR_GFld_GetSpatialRef").into());
        }
        SpatialRef::from_c_obj(c_obj)
    }
}
