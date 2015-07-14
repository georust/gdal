use libc::c_int;
use utils::_string;
use vector::ogr;

/// Layer definition
///
/// Defines the fields available for features in a layer.
pub struct Defn {
    c_defn: *const (),
}

impl Defn {
    pub unsafe fn _with_c_defn(c_defn: *const ()) -> Defn {
        Defn{c_defn: c_defn}
    }

    pub unsafe fn c_defn(&self) -> *const () { self.c_defn }

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
}

pub struct FieldIterator<'a> {
    defn: &'a Defn,
    c_feature_defn: *const (),
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
    c_field_defn: *const (),
}

impl<'a> Field<'a> {
    /// Get the name of this field.
    pub fn name(&'a self) -> String {
        let rv = unsafe { ogr::OGR_Fld_GetNameRef(self.c_field_defn) };
        return _string(rv);
    }
}
