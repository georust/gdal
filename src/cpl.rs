//! GDAL Common Portability Library Functions
//!
//! This module provides safe access to a subset of the [GDAL CPL functions](https://gdal.org/api/cpl.html).
//!

use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::ptr;

use gdal_sys::{CSLCount, CSLDestroy, CSLDuplicate, CSLFetchNameValue, CSLSetNameValue};
use libc::c_char;

use crate::errors::{GdalError, Result};
use crate::utils::{_string, _string_tuple};

/// Wraps a [`gdal_sys::CSLConstList`]  (a.k.a. `char **papszStrList`). This data structure
/// (a null-terminated array of null-terminated strings) is used throughout GDAL to pass
/// `KEY=VALUE`-formatted options to various functions.
///
/// See the [`CSL*` GDAL functions](https://gdal.org/api/cpl.html#cpl-string-h) for more details.
pub struct CslStringList {
    list_ptr: *mut *mut c_char,
}

impl CslStringList {
    /// Creates an empty GDAL string list.
    pub fn new() -> Self {
        Self {
            list_ptr: ptr::null_mut(),
        }
    }

    /// Assigns `value` to `name`.
    ///
    /// Overwrites duplicate `name`s.
    ///
    /// Returns `Ok<()>` on success, `Err<GdalError>` if `name` has non alphanumeric
    /// characters, or `value` has newline characters.
    pub fn set_name_value(&mut self, name: &str, value: &str) -> Result<()> {
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(GdalError::BadArgument(format!(
                "Invalid characters in name: '{}'",
                name
            )));
        }
        if value.contains(|c| c == '\n' || c == '\r') {
            return Err(GdalError::BadArgument(format!(
                "Invalid characters in value: '{}'",
                value
            )));
        }
        let psz_name = CString::new(name)?;
        let psz_value = CString::new(value)?;

        unsafe {
            self.list_ptr = CSLSetNameValue(self.list_ptr, psz_name.as_ptr(), psz_value.as_ptr());
        }

        Ok(())
    }

    /// Looks up the value corresponding to `key`.
    ///
    /// See [`CSLFetchNameValue`](https://gdal.org/doxygen/cpl__string_8h.html#a4f23675f8b6f015ed23d9928048361a1)
    /// for details.
    pub fn fetch_name_value(&self, key: &str) -> Result<Option<String>> {
        let key = CString::new(key)?;
        let c_value = unsafe { CSLFetchNameValue(self.as_ptr(), key.as_ptr()) };
        let value = if c_value.is_null() {
            None
        } else {
            Some(_string(c_value))
        };
        Ok(value)
    }

    /// Determine the number of entries in the list.
    pub fn len(&self) -> usize {
        (unsafe { CSLCount(self.as_ptr()) }) as usize
    }

    /// Determine if the list has any values
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the name/value elements of the list.
    pub fn iter(&self) -> CslStringListIterator {
        CslStringListIterator::new(self)
    }

    /// Get the raw pointer to the underlying data.
    pub fn as_ptr(&self) -> gdal_sys::CSLConstList {
        self.list_ptr
    }
}

impl Drop for CslStringList {
    fn drop(&mut self) {
        unsafe { CSLDestroy(self.list_ptr) }
    }
}

impl Default for CslStringList {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CslStringList {
    fn clone(&self) -> Self {
        let list_ptr = unsafe { CSLDuplicate(self.list_ptr) };
        Self { list_ptr }
    }
}

/// State for iterator over [`CslStringList`] entries.
pub struct CslStringListIterator<'a> {
    list: &'a CslStringList,
    idx: usize,
    count: usize,
}

impl<'a> CslStringListIterator<'a> {
    fn new(list: &'a CslStringList) -> Self {
        Self {
            list,
            idx: 0,
            count: list.len(),
        }
    }
    fn is_done(&self) -> bool {
        self.idx >= self.count
    }
}

impl<'a> Iterator for CslStringListIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done() {
            return None;
        }

        let field = unsafe {
            // Equivalent to, but less traversals than:
            // CSLGetField(self.list.as_ptr(), self.idx as libc::c_int)
            let slice = std::slice::from_raw_parts(self.list.list_ptr, self.count);
            slice[self.idx]
        };
        if field.is_null() {
            None
        } else {
            self.idx += 1;
            _string_tuple(field, '=')
        }
    }
}

impl Debug for CslStringList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.iter() {
            f.write_fmt(format_args!("{k}={v}\n"))?;
        }
        Ok(())
    }
}

/// Convenience shorthand for specifying an empty `CslStringList` to functions accepting
/// `Into<CslStringList>`.
impl From<()> for CslStringList {
    fn from(_: ()) -> Self {
        CslStringList::default()
    }
}

/// Creates a [`CslStringList`] from a slice of _key_/_value_ tuples.
impl<const N: usize> From<&[(&str, &str); N]> for CslStringList {
    fn from(pairs: &[(&str, &str); N]) -> Self {
        let mut result = Self::default();
        for (k, v) in pairs {
            result.set_name_value(k, v).expect("valid key/value pair");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::cpl::CslStringList;
    use crate::errors::Result;

    fn fixture() -> Result<CslStringList> {
        let mut l = CslStringList::new();
        l.set_name_value("ONE", "1")?;
        l.set_name_value("TWO", "2")?;
        l.set_name_value("THREE", "3")?;

        Ok(l)
    }

    #[test]
    fn basic_list() -> Result<()> {
        let l = fixture()?;
        assert!(matches!(l.fetch_name_value("ONE"), Ok(Some(s)) if s == *"1"));
        assert!(matches!(l.fetch_name_value("THREE"), Ok(Some(s)) if s == *"3"));
        assert!(matches!(l.fetch_name_value("FOO"), Ok(None)));

        Ok(())
    }

    #[test]
    fn has_length() -> Result<()> {
        let l = fixture()?;
        assert_eq!(l.len(), 3);

        Ok(())
    }

    #[test]
    fn can_be_empty() -> Result<()> {
        let l = CslStringList::new();
        assert!(l.is_empty());

        let l = fixture()?;
        assert!(!l.is_empty());

        Ok(())
    }

    #[test]
    fn has_iterator() -> Result<()> {
        let f = fixture()?;
        let mut it = f.iter();
        assert_eq!(it.next(), Some(("ONE".to_string(), "1".to_string())));
        assert_eq!(it.next(), Some(("TWO".to_string(), "2".to_string())));
        assert_eq!(it.next(), Some(("THREE".to_string(), "3".to_string())));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
        Ok(())
    }

    #[test]
    fn invalid_keys() -> Result<()> {
        let mut l = fixture()?;
        assert!(l.set_name_value("l==t", "2").is_err());
        assert!(l.set_name_value("foo", "2\n4\r5").is_err());

        Ok(())
    }

    #[test]
    fn debug_fmt() -> Result<()> {
        let l = fixture()?;
        let s = format!("{l:?}");
        assert!(s.contains("ONE=1"));
        assert!(s.contains("TWO=2"));
        assert!(s.contains("THREE=3"));

        Ok(())
    }
}
