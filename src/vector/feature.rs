use libc::c_char;
use std::ptr::null;
use std::ffi::CString;
use vector::Layer;
use utils::_string;
use vector::{ogr, Geometry, FeatureGeometry};


pub struct Feature<'a> {
    _layer: &'a Layer<'a>,
    c_feature: *const (),
}


impl<'a> Feature<'a> {
    pub unsafe fn _with_layer(layer: &'a Layer<'a>, c_feature: *const ()) -> Feature<'a> {
        return Feature{_layer: layer, c_feature: c_feature};
    }

    pub fn field(&self, name: &str) -> Option<FieldValue> {
        let c_name = CString::from_slice(name.as_bytes());
        let field_id = unsafe { ogr::OGR_F_GetFieldIndex(self.c_feature, c_name.as_ptr()) };
        if field_id == -1 {
            return None;
        }
        let field_defn = unsafe { ogr::OGR_F_GetFieldDefnRef(self.c_feature, field_id) };
        let field_type = unsafe { ogr::OGR_Fld_GetType(field_defn) };
        return match field_type {
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

    pub fn geometry(&'a self) -> FeatureGeometry<'a> {
        let c_geometry = unsafe { ogr::OGR_F_GetGeometryRef(self.c_feature) };
        return unsafe { FeatureGeometry::with_ref(c_geometry, self) };
    }
}


#[unsafe_destructor]
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
    pub fn as_string(self) -> String {
        match self {
            FieldValue::StringValue(rv) => rv,
            _ => panic!("not a StringValue")
        }
    }

    pub fn as_real(self) -> f64 {
        match self {
            FieldValue::RealValue(rv) => rv,
            _ => panic!("not a RealValue")
        }
    }
}
