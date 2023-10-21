//! GDAL Common Portability Library Functions
//!
//! This module provides safe access to a subset of the [GDAL CPL functions](https://gdal.org/api/cpl.html).
//!

use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::ptr;

use gdal_sys::{CSLAddString, CSLCount, CSLDestroy, CSLDuplicate, CSLFetchNameValue, CSLFindString, CSLFindStringCaseSensitive, CSLGetField, CSLPartialFindString, CSLSetNameValue};
use libc::{c_char, c_int};

use crate::errors::{GdalError, Result};
use crate::utils::_string;

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
                "Invalid characters in name: '{name}'"
            )));
        }
        if value.contains(|c| c == '\n' || c == '\r') {
            return Err(GdalError::BadArgument(format!(
                "Invalid characters in value: '{value}'"
            )));
        }
        let psz_name = CString::new(name)?;
        let psz_value = CString::new(value)?;

        unsafe {
            self.list_ptr = CSLSetNameValue(self.list_ptr, psz_name.as_ptr(), psz_value.as_ptr());
        }

        Ok(())
    }

    /// Adds a copy of the string slice `value` to the list.
    ///
    /// Returns `Ok<()>` on success, `Err<GdalError>` if `value` cannot be converted to a C string,
    /// e.g. `value` contains a `0` byte, which is used as a string termination sentinel in C.
    ///
    /// See: [`CSLAddString`](https://gdal.org/api/cpl.html#_CPPv412CSLAddStringPPcPKc)
    pub fn add_string(&mut self, value: &str) -> Result<()> {
        let v = CString::new(value)?;
        self.list_ptr = unsafe { CSLAddString(self.list_ptr, v.as_ptr()) };
        Ok(())
    }

    /// Looks up the value corresponding to `key`.
    ///
    /// See [`CSLFetchNameValue`](https://gdal.org/doxygen/cpl__string_8h.html#a4f23675f8b6f015ed23d9928048361a1)
    /// for details.
    pub fn fetch_name_value(&self, key: &str) -> Option<String> {
        // If CString conversion fails because `key` has an embedded null byte, then
        // we know already `key` will never exist in a valid CslStringList.
        let key = CString::new(key).ok()?;
        let c_value = unsafe { CSLFetchNameValue(self.as_ptr(), key.as_ptr()) };
        if c_value.is_null() {
            None
        } else {
            Some(_string(c_value))
        }
    }

    /// Perform a case <u>insensitive</u> search for the given string
    ///
    /// Returns `Some(usize)` of value index position, or `None` if not found.
    ///
    /// See: [`CSLFindString`](https://gdal.org/api/cpl.html#_CPPv413CSLFindString12CSLConstListPKc)
    /// for details.
    pub fn find_string(&self, value: &str) -> Option<usize> {
        let value = CString::new(value).ok()?;
        let idx = unsafe { CSLFindString(self.as_ptr(), value.as_ptr()) };
        if idx < 0 {
            None
        } else {
            Some(idx as usize)
        }
    }

    /// Perform a case sensitive search for the given string
    ///
    /// Returns `Some(usize)` of value index position, or `None` if not found.
    ///
    /// See: [`CSLFindString`](https://gdal.org/api/cpl.html#_CPPv413CSLFindString12CSLConstListPKc)
    /// for details.
    pub fn find_string_case_sensitive(&self, value: &str) -> Option<usize> {
        let value = CString::new(value).ok()?;
        let idx = unsafe { CSLFindStringCaseSensitive(self.as_ptr(), value.as_ptr()) };
        if idx < 0 {
            None
        } else {
            Some(idx as usize)
        }
    }

    /// Perform a case sensitive partial string search indicated by `fragment`.
    ///
    /// Returns `Some(usize)` of value index position, or `None` if not found.
    ///
    /// See:: [`CSLPartialFindString`](https://gdal.org/api/cpl.html#_CPPv420CSLPartialFindString12CSLConstListPKc)
    /// for details.
    pub fn partial_find_string(&self, fragment: &str) -> Option<usize> {
        let fragment = CString::new(fragment).ok()?;
        let idx = unsafe { CSLPartialFindString(self.as_ptr(), fragment.as_ptr()) };
        if idx < 0 {
            None
        } else {
            Some(idx as usize)
        }
    }

    /// Fetch the [CslStringListEntry] for the entry at the given index.
    ///
    /// Returns `None` if index is out of bounds
    pub fn get_field(&self, index: usize) -> Option<CslStringListEntry> {
        // In the C++ implementation, an index-out-of-bounds returns an empty string, not an error.
        // We don't want to check against `len` because that scans the list.
        // See: https://github.com/OSGeo/gdal/blob/fada29feb681e97f0fc4e8861e07f86b16855681/port/cpl_string.cpp#L181-L182
        let field = unsafe { CSLGetField(self.as_ptr(), index as c_int) };
        if field.is_null() {
            return None
        }

        let field = _string(field);
        if field.is_empty() {
            None
        }
        else {
            Some(field.deref().into())
        }
    }

    /// Determine the number of entries in the list.
    ///
    /// See [`CSLCount`](https://gdal.org/api/cpl.html#_CPPv48CSLCount12CSLConstList) for details.
    pub fn len(&self) -> usize {
        (unsafe { CSLCount(self.as_ptr()) }) as usize
    }

    /// Determine if the list has any values
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the `name=value` elements of the list.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=CslStringListEntry> + 'a {
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

impl Debug for CslStringList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut b = f.debug_tuple("CslStringList");

        for e in self.iter() {
            b.field(&e.to_string());
        }

        b.finish()
    }
}

/// Represents an entry in a [CslStringList], which is ether a single token, or a key/value assignment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CslStringListEntry {
    Arg(String),
    Assign { key: String, value: String },
}

impl From<&str> for CslStringListEntry {
    fn from(value: &str) -> Self {
        match value.split_once('=') {
            Some(kv) => kv.into(),
            None => Self::Arg(value.to_owned()),
        }
    }
}

impl From<(&str, &str)> for CslStringListEntry {
    fn from((key, value): (&str, &str)) -> Self {
        Self::Assign {
            key: key.to_owned(),
            value: value.to_owned(),
        }
    }
}

impl Display for CslStringListEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CslStringListEntry::Arg(s) => f.write_str(s),
            CslStringListEntry::Assign { key, value } => f.write_fmt(format_args!("{key}={value}")),
        }
    }
}

/// State for iterator over [`CslStringList`] entries.
///
/// Note: Does not include values inserted with [CslStringList::add_string]
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
    type Item = CslStringListEntry;

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
        self.idx += 1;
        if field.is_null() {
            None
        } else {
            let entry = _string(field);
            Some(entry.deref().into())
        }
    }
}

/// Convenience for creating a [`CslStringList`] from a slice of _key_/_value_ tuples.
///
/// # Example
///
/// ```rust, no_run
/// use gdal::cpl::CslStringList;
///
/// let opts = CslStringList::try_from(&[("One", "1"), ("Two", "2"), ("Three", "3")]).expect("known valid");
/// assert_eq!(opts.len(), 3);
/// ```
impl<const N: usize> TryFrom<&[(&str, &str); N]> for CslStringList {
    type Error = GdalError;

    fn try_from(pairs: &[(&str, &str); N]) -> Result<Self> {
        let mut result = Self::default();
        for (k, v) in pairs {
            result.set_name_value(k, v)?;
        }
        Ok(result)
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
        l.add_string("SOME_FLAG")?;
        Ok(l)
    }

    #[test]
    fn basic_list() -> Result<()> {
        let l = fixture()?;
        assert!(matches!(l.fetch_name_value("ONE"), Some(s) if s == *"1"));
        assert!(matches!(l.fetch_name_value("THREE"), Some(s) if s == *"3"));
        assert!(matches!(l.fetch_name_value("FOO"), None));

        Ok(())
    }

    #[test]
    fn has_length() -> Result<()> {
        let l = fixture()?;
        assert_eq!(l.len(), 4);

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
        assert_eq!(it.next(), Some(("ONE", "1").into()));
        assert_eq!(it.next(), Some(("TWO", "2").into()));
        assert_eq!(it.next(), Some(("THREE", "3").into()));
        assert_eq!(it.next(), Some("SOME_FLAG".into()));
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
    fn try_from_impl() -> Result<()> {
        let l = CslStringList::try_from(&[("ONE", "1"), ("TWO", "2")])?;
        assert!(matches!(l.fetch_name_value("ONE"), Some(s) if s == *"1"));
        assert!(matches!(l.fetch_name_value("TWO"), Some(s) if s == *"2"));

        Ok(())
    }

    #[test]
    fn debug_fmt() -> Result<()> {
        let l = fixture()?;
        let s = format!("{l:?}");
        assert!(s.contains("ONE=1"));
        assert!(s.contains("TWO=2"));
        assert!(s.contains("THREE=3"));
        assert!(s.contains("SOME_FLAG"));

        Ok(())
    }

    #[test]
    fn can_add_strings() -> Result<()> {
        let mut l = CslStringList::new();
        assert!(l.is_empty());
        l.add_string("-abc")?;
        l.add_string("-d_ef")?;
        l.add_string("A")?;
        l.add_string("B")?;
        assert_eq!(l.len(), 4);

        Ok(())
    }

    #[test]
    fn find_string() -> Result<()> {
        let f = fixture()?;
        assert_eq!(f.find_string("NON_FLAG"), None);
        assert_eq!(f.find_string("SOME_FLAG"), Some(3));
        assert_eq!(f.find_string("ONE=1"), Some(0));
        assert_eq!(f.find_string("one=1"), Some(0));
        assert_eq!(f.find_string("TWO="), None);
        Ok(())
    }

    #[test]
    fn find_string_case_sensitive() -> Result<()> {
        let f = fixture()?;
        assert_eq!(f.find_string_case_sensitive("ONE=1"), Some(0));
        assert_eq!(f.find_string_case_sensitive("one=1"), None);
        assert_eq!(f.find_string_case_sensitive("SOME_FLAG"), Some(3));
        Ok(())
    }

    #[test]
    fn partial_find_string() -> Result<()> {
        let f = fixture()?;
        assert_eq!(f.partial_find_string("ONE=1"), Some(0));
        assert_eq!(f.partial_find_string("ONE="), Some(0));
        assert_eq!(f.partial_find_string("=1"), Some(0));
        assert_eq!(f.partial_find_string("1"), Some(0));
        assert_eq!(f.partial_find_string("THREE="), Some(2));
        assert_eq!(f.partial_find_string("THREE"), Some(2));
        assert_eq!(f.partial_find_string("three"), None);
        Ok(())
    }
}
