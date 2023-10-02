use crate::spatial_ref::{SpatialRef, SpatialRefRef};
use crate::utils::{_last_null_pointer_err, _string};
use crate::vector::LayerAccess;
use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};
use gdal_sys::{
    self, OGRFeatureDefnH, OGRFieldDefnH, OGRFieldType, OGRGeomFieldDefnH, OGRwkbGeometryType,
};
use libc::c_int;
use std::fmt::{Debug, Formatter};

use crate::errors::*;

foreign_type! {
    /// Layer definition
    ///
    /// Defines the fields available for features in a layer.
    pub unsafe type Defn {
        type CType = libc::c_void;
        fn drop = gdal_sys::OGR_FD_Release;
    }
}

impl Defn {
    pub fn from_layer<L: LayerAccess>(lyr: &L) -> Self {
        DefnRef::from_layer(lyr).to_owned()
    }
}

/// GDAL implements reference counting over `OGRFeatureDefn`, so
/// we can implement cheaper ownership via reference counting.
impl ToOwned for DefnRef {
    type Owned = Defn;

    fn to_owned(&self) -> Self::Owned {
        let ptr = self.as_ptr();
        let _ = unsafe { gdal_sys::OGR_FD_Reference(ptr) };
        unsafe { Defn::from_ptr(ptr) }
    }
}

impl DefnRef {
    pub fn from_layer<L: LayerAccess>(lyr: &L) -> &DefnRef {
        unsafe { DefnRef::from_ptr(gdal_sys::OGR_L_GetLayerDefn(lyr.c_layer())) }
    }

    /// Number of non-geometry fields in the feature definition
    ///
    /// See: [`OGR_FD_GetFieldCount`](https://gdal.org/api/vector_c_api.html#_CPPv420OGR_FD_GetFieldCount15OGRFeatureDefnH)
    pub fn field_count(&self) -> isize {
        (unsafe { gdal_sys::OGR_FD_GetFieldCount(self.as_ptr()) } as isize)
    }

    /// Iterate over the field schema of this layer.
    pub fn fields(&self) -> FieldIterator {
        let total = self.field_count();
        FieldIterator {
            defn: self,
            c_feature_defn: self.as_ptr(),
            next_id: 0,
            total,
        }
    }

    /// Number of geometry fields in the feature definition
    ///
    /// See: [`OGR_FD_GetGeomFieldCount`](https://gdal.org/api/vector_c_api.html#_CPPv424OGR_FD_GetGeomFieldCount15OGRFeatureDefnH)
    pub fn geom_field_count(&self) -> isize {
        (unsafe { gdal_sys::OGR_FD_GetGeomFieldCount(self.as_ptr()) } as isize)
    }

    /// Iterate over the geometry field schema of this layer.
    pub fn geom_fields(&self) -> GeomFieldIterator {
        let total = self.geom_field_count();
        GeomFieldIterator {
            defn: self,
            c_feature_defn: self.as_ptr(),
            next_id: 0,
            total,
        }
    }
}

impl Debug for Defn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let f_count = self.field_count();
        let g_count = self.geom_fields().count();
        f.debug_struct("Defn")
            .field("fields", &f_count)
            .field("geometries", &g_count)
            .finish()
    }
}

pub struct FieldIterator<'a> {
    defn: &'a DefnRef,
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
    _defn: &'a DefnRef,
    c_field_defn: OGRFieldDefnH,
}

impl<'a> Field<'a> {
    /// Get the name of this field.
    pub fn name(&'a self) -> String {
        let rv = unsafe { gdal_sys::OGR_Fld_GetNameRef(self.c_field_defn) };
        _string(rv)
    }

    pub fn field_type(&'a self) -> OGRFieldType::Type {
        unsafe { gdal_sys::OGR_Fld_GetType(self.c_field_defn) }
    }

    pub fn width(&'a self) -> i32 {
        unsafe { gdal_sys::OGR_Fld_GetWidth(self.c_field_defn) }
    }

    pub fn precision(&'a self) -> i32 {
        unsafe { gdal_sys::OGR_Fld_GetPrecision(self.c_field_defn) }
    }
}

pub struct GeomFieldIterator<'a> {
    defn: &'a DefnRef,
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
    _defn: &'a DefnRef,
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
        Ok(unsafe { SpatialRefRef::from_ptr(c_obj).to_owned() })
    }
}
