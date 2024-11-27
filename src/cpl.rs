//! GDAL Common Portability Library Functions
//!
//! This module provides safe access to a subset of the [GDAL CPL functions](https://gdal.org/api/cpl.html).
//!

use std::ffi::{c_char, c_int, CString};
use std::fmt::{Debug, Display, Formatter};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr;
use std::str::FromStr;

use gdal_sys::{
    CSLAddNameValue, CSLAddString, CSLCount, CSLDestroy, CSLDuplicate, CSLFetchNameValue,
    CSLFindString, CSLFindStringCaseSensitive, CSLGetField, CSLPartialFindString, CSLSetNameValue,
    CSLTokenizeString2,
};

use crate::errors::{GdalError, Result};
use crate::utils::_string;

/// Wraps a [`gdal_sys::CSLConstList`]  (a.k.a. `char **papszStrList`).
///
/// This data structure (a null-terminated array of null-terminated strings) is used throughout
/// GDAL to pass `KEY=VALUE`-formatted options to various functions.
///
/// # Example
///
/// There are a number of ways to populate a [`CslStringList`]:
///
/// ```rust, no_run
/// use gdal::cpl::{CslStringList, CslStringListEntry};
///
/// let mut sl1 = CslStringList::new();
/// sl1.set_name_value("NUM_THREADS", "ALL_CPUS").unwrap();
/// sl1.set_name_value("COMPRESS", "LZW").unwrap();
/// sl1.add_string("MAGIC_FLAG").unwrap();
///
/// let sl2: CslStringList = "NUM_THREADS=ALL_CPUS COMPRESS=LZW MAGIC_FLAG".parse().unwrap();
/// let sl3 = CslStringList::from_iter(["NUM_THREADS=ALL_CPUS", "COMPRESS=LZW", "MAGIC_FLAG"]);
/// let sl4 = CslStringList::from_iter([
///     CslStringListEntry::from(("NUM_THREADS", "ALL_CPUS")),
///     CslStringListEntry::from(("COMPRESS", "LZW")),
///     CslStringListEntry::from("MAGIC_FLAG")
/// ]);
///
/// assert_eq!(sl1.to_string(), sl2.to_string());
/// assert_eq!(sl2.to_string(), sl3.to_string());
/// assert_eq!(sl3.to_string(), sl4.to_string());
/// ```
/// One [`CslStringList`] can be combined with another:
///
/// ```rust
/// use gdal::cpl::CslStringList;
/// let mut base: CslStringList = "NUM_THREADS=ALL_CPUS COMPRESS=LZW".parse().unwrap();
/// let debug: CslStringList = "CPL_CURL_VERBOSE=YES CPL_DEBUG=YES".parse().unwrap();
/// base.extend(&debug);
///
/// assert_eq!(base.fetch_name_value("CPL_DEBUG"), Some("YES".into()));
/// ```
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

    /// Check that the given `name` is a valid [`CslStringList`] key.
    ///
    /// Per [GDAL documentation](https://gdal.org/api/cpl.html#_CPPv415CSLSetNameValuePPcPKcPKc),
    /// a key cannot have non-alphanumeric characters in it.
    ///
    /// Returns `Err(GdalError::BadArgument)` on invalid name, `Ok(())` otherwise.
    fn check_valid_name(name: &str) -> Result<()> {
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            Err(GdalError::BadArgument(format!(
                "Invalid characters in name: '{name}'"
            )))
        } else {
            Ok(())
        }
    }

    /// Check that the given `value` is a valid [`CslStringList`] value.
    ///
    /// Per [GDAL documentation](https://gdal.org/api/cpl.html#_CPPv415CSLSetNameValuePPcPKcPKc),
    /// a key cannot have newline characters in it.
    ///
    /// Returns `Err(GdalError::BadArgument)` on invalid value, `Ok(())` otherwise.
    fn check_valid_value(value: &str) -> Result<()> {
        if value.contains(['\n', '\r']) {
            Err(GdalError::BadArgument(format!(
                "Invalid characters in value: '{value}'"
            )))
        } else {
            Ok(())
        }
    }

    /// Assigns `value` to the key `name` without checking for a pre-existing assignments.
    ///
    /// Returns `Ok(())` on success, or `Err(GdalError::BadArgument)`
    /// if `name` has non-alphanumeric characters or `value` has newline characters.
    ///
    /// See: [`CSLAddNameValue`](https://gdal.org/api/cpl.html#_CPPv415CSLAddNameValuePPcPKcPKc)
    /// for details.
    pub fn add_name_value(&mut self, name: &str, value: &str) -> Result<()> {
        Self::check_valid_name(name)?;
        Self::check_valid_value(value)?;

        let psz_name = CString::new(name)?;
        let psz_value = CString::new(value)?;

        unsafe {
            self.list_ptr = CSLAddNameValue(self.list_ptr, psz_name.as_ptr(), psz_value.as_ptr());
        }

        Ok(())
    }

    /// Assigns `value` to the key `name`, overwriting any existing assignment to `name`.
    ///
    /// Returns `Ok(())` on success, or `Err(GdalError::BadArgument)`
    /// if `name` has non-alphanumeric characters or `value` has newline characters.
    ///
    /// See: [`CSLSetNameValue`](https://gdal.org/api/cpl.html#_CPPv415CSLSetNameValuePPcPKcPKc)
    /// for details.
    pub fn set_name_value(&mut self, name: &str, value: &str) -> Result<()> {
        Self::check_valid_name(name)?;
        Self::check_valid_value(value)?;

        let psz_name = CString::new(name)?;
        let psz_value = CString::new(value)?;

        unsafe {
            self.list_ptr = CSLSetNameValue(self.list_ptr, psz_name.as_ptr(), psz_value.as_ptr());
        }

        Ok(())
    }

    /// Adds a copy of the string slice `value` to the list.
    ///
    /// Returns `Ok(())` on success, `Err(GdalError::FfiNulError)` if `value` cannot be converted to a C string,
    /// e.g. `value` contains a `0` byte, which is used as a string termination sentinel in C.
    ///
    /// See: [`CSLAddString`](https://gdal.org/api/cpl.html#_CPPv412CSLAddStringPPcPKc)
    pub fn add_string(&mut self, value: &str) -> Result<()> {
        let v = CString::new(value)?;
        self.list_ptr = unsafe { CSLAddString(self.list_ptr, v.as_ptr()) };
        Ok(())
    }

    /// Adds the contents of a [`CslStringListEntry`] to `self`.
    ///
    /// Returns `Err(GdalError::BadArgument)` if entry doesn't not meet entry restrictions as
    /// described by [`CslStringListEntry`].
    pub fn add_entry(&mut self, entry: &CslStringListEntry) -> Result<()> {
        match entry {
            CslStringListEntry::Flag(f) => self.add_string(f),
            CslStringListEntry::Pair { name, value } => self.add_name_value(name, value),
        }
    }

    /// Looks up the value corresponding to `name`.
    ///
    /// See [`CSLFetchNameValue`](https://gdal.org/doxygen/cpl__string_8h.html#a4f23675f8b6f015ed23d9928048361a1)
    /// for details.
    pub fn fetch_name_value(&self, name: &str) -> Option<String> {
        // If CString conversion fails because `key` has an embedded null byte, then
        // we know already `name` will never exist in a valid CslStringList.
        let key = CString::new(name).ok()?;
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
    /// See: [`CSLPartialFindString`](https://gdal.org/api/cpl.html#_CPPv420CSLPartialFindString12CSLConstListPKc)
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

    /// Fetch the [`CslStringListEntry`] for the entry at the given index.
    ///
    /// Returns `None` if index is out of bounds, `Some(entry)` otherwise.
    pub fn get_field(&self, index: usize) -> Option<CslStringListEntry> {
        if index > c_int::MAX as usize {
            return None;
        }
        // In the C++ implementation, an index-out-of-bounds returns an empty string, not an error.
        // We don't want to check against `len` because that scans the list.
        // See: https://github.com/OSGeo/gdal/blob/fada29feb681e97f0fc4e8861e07f86b16855681/port/cpl_string.cpp#L181-L182
        let field = unsafe { CSLGetField(self.as_ptr(), index as c_int) };
        if field.is_null() {
            return None;
        }

        let field = _string(field);
        if field.is_empty() {
            None
        } else {
            Some(field.deref().into())
        }
    }

    /// Determine the number of entries in the list.
    ///
    /// See: [`CSLCount`](https://gdal.org/api/cpl.html#_CPPv48CSLCount12CSLConstList) for details.
    pub fn len(&self) -> usize {
        (unsafe { CSLCount(self.as_ptr()) }) as usize
    }

    /// Determine if the list has any values
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the `name=value` elements of the list.
    pub fn iter(&self) -> CslStringListIterator {
        CslStringListIterator::new(self)
    }

    /// Get the raw pointer to the underlying data.
    pub fn as_ptr(&self) -> gdal_sys::CSLConstList {
        self.list_ptr
    }

    /// Get the raw pointer to the underlying data, passing ownership
    /// (and responsibility for freeing) to the the caller.
    pub fn into_ptr(self) -> gdal_sys::CSLConstList {
        let s = ManuallyDrop::new(self);
        s.list_ptr
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

impl Display for CslStringList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // CSLPrint would be preferable here, but it can only write to a file descriptor.
        for e in self.iter() {
            f.write_fmt(format_args!("{e}\n"))?;
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a CslStringList {
    type Item = CslStringListEntry;
    type IntoIter = CslStringListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Parse a space-delimited string into a [`CslStringList`].
///
/// See [`CSLTokenizeString`](https://gdal.org/api/cpl.html#_CPPv417CSLTokenizeStringPKc) for details
impl FromStr for CslStringList {
    type Err = GdalError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // See: https://github.com/OSGeo/gdal/blob/cd2a054b0d7b881534baece69a8f52ddb69a53d9/port/cpl_string.h#L86C1-L97
        static CSLT_HONOURSTRINGS: c_int = 0x0001;
        static CSLT_PRESERVEESCAPES: c_int = 0x0008;
        static DELIM: &[u8; 4] = b" \n\t\0";

        let cstr = CString::new(s)?;
        let c_list = unsafe {
            CSLTokenizeString2(
                cstr.as_ptr(),
                DELIM.as_ptr() as *const c_char,
                CSLT_HONOURSTRINGS | CSLT_PRESERVEESCAPES,
            )
        };
        Ok(Self { list_ptr: c_list })
    }
}

impl FromIterator<CslStringListEntry> for CslStringList {
    fn from_iter<T: IntoIterator<Item = CslStringListEntry>>(iter: T) -> Self {
        let mut result = Self::default();
        for e in iter {
            result.add_entry(&e).unwrap_or_default()
        }
        result
    }
}

impl<'a> FromIterator<&'a str> for CslStringList {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        iter.into_iter()
            .map(Into::<CslStringListEntry>::into)
            .collect()
    }
}

impl FromIterator<String> for CslStringList {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        iter.into_iter()
            .map(Into::<CslStringListEntry>::into)
            .collect()
    }
}

impl Extend<CslStringListEntry> for CslStringList {
    fn extend<T: IntoIterator<Item = CslStringListEntry>>(&mut self, iter: T) {
        for e in iter {
            self.add_entry(&e).unwrap_or_default();
        }
    }
}

/// Represents an entry in a [`CslStringList`]
///
/// An Entry is ether a single token (`Flag`), or a `name=value` assignment (`Pair`).
///
/// Note: When constructed directly, assumes string values do not contain newline characters nor
/// the null `\0` character. If these conditions are violated, the provided values will be ignored.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CslStringListEntry {
    /// A single token entry.
    Flag(String),
    /// A `name=value` pair entry.
    Pair { name: String, value: String },
}

impl CslStringListEntry {
    /// Create a new [`Self::Flag`] entry.
    ///
    /// Assumes `flag` does not contain `=`, `\n`, `\r`, or `\0`.  If it does, an
    /// error will be returned by [`CslStringList::add_entry`].
    pub fn new_flag(flag: &str) -> Self {
        CslStringListEntry::Flag(flag.to_owned())
    }

    /// Create a new [`Self::Pair`] entry.
    ///
    /// Assumes neither `name` nor `value` contain `=`, `\n`, `\r`, or `\0`.  If it does, an
    /// error will be returned by [`CslStringList::add_entry`].
    pub fn new_pair(name: &str, value: &str) -> Self {
        CslStringListEntry::Pair {
            name: name.to_owned(),
            value: value.to_owned(),
        }
    }
}

impl From<&str> for CslStringListEntry {
    fn from(value: &str) -> Self {
        // `into` parses for '='
        value.to_owned().into()
    }
}

impl From<(&str, &str)> for CslStringListEntry {
    fn from((key, value): (&str, &str)) -> Self {
        Self::new_pair(key, value)
    }
}

impl From<String> for CslStringListEntry {
    fn from(value: String) -> Self {
        match value.split_once('=') {
            Some((name, value)) => Self::new_pair(name, value),
            None => Self::new_flag(&value),
        }
    }
}

impl From<(String, String)> for CslStringListEntry {
    fn from((name, value): (String, String)) -> Self {
        // Using struct initializer rather than method to avoid
        // going to/from slice.
        Self::Pair { name, value }
    }
}

impl Display for CslStringListEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CslStringListEntry::Flag(s) => f.write_str(s),
            CslStringListEntry::Pair { name: key, value } => {
                f.write_fmt(format_args!("{key}={value}"))
            }
        }
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

impl Iterator for CslStringListIterator<'_> {
    type Item = CslStringListEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done() {
            return None;
        }

        let field = unsafe {
            // Equivalent to, but fewer traversals than:
            // CSLGetField(self.list.as_ptr(), self.idx as c_int)
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

#[cfg(test)]
mod tests {
    use super::*;
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
    fn construct() -> Result<()> {
        let mut sl1 = CslStringList::new();
        sl1.set_name_value("NUM_THREADS", "ALL_CPUS").unwrap();
        sl1.set_name_value("COMPRESS", "LZW").unwrap();
        sl1.add_string("MAGIC_FLAG").unwrap();

        let sl2: CslStringList = "NUM_THREADS=ALL_CPUS COMPRESS=LZW MAGIC_FLAG"
            .parse()
            .unwrap();
        let sl3 = CslStringList::from_iter(["NUM_THREADS=ALL_CPUS", "COMPRESS=LZW", "MAGIC_FLAG"]);
        let sl4 = CslStringList::from_iter([
            CslStringListEntry::from(("NUM_THREADS", "ALL_CPUS")),
            CslStringListEntry::from(("COMPRESS", "LZW")),
            CslStringListEntry::from("MAGIC_FLAG"),
        ]);

        assert_eq!(sl1.to_string(), sl2.to_string());
        assert_eq!(sl2.to_string(), sl3.to_string());
        assert_eq!(sl3.to_string(), sl4.to_string());

        Ok(())
    }

    #[test]
    fn basic_list() -> Result<()> {
        let l = fixture()?;
        assert!(matches!(l.fetch_name_value("ONE"), Some(s) if s == *"1"));
        assert!(matches!(l.fetch_name_value("THREE"), Some(s) if s == *"3"));
        assert!(l.fetch_name_value("FOO").is_none());

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
    fn invalid_name_value() -> Result<()> {
        let mut l = fixture()?;
        assert!(l.set_name_value("l==t", "2").is_err());
        assert!(l.set_name_value("foo", "2\n4\r5").is_err());

        Ok(())
    }

    #[test]
    fn add_vs_set() -> Result<()> {
        let mut f = CslStringList::new();
        f.add_name_value("ONE", "1")?;
        f.add_name_value("ONE", "2")?;
        let s = f.to_string();
        assert!(s.contains("ONE") && s.contains('1') && s.contains('2'));

        let mut f = CslStringList::new();
        f.set_name_value("ONE", "1")?;
        f.set_name_value("ONE", "2")?;
        let s = f.to_string();
        assert!(s.contains("ONE") && !s.contains('1') && s.contains('2'));

        Ok(())
    }

    #[test]
    fn try_from_impl() -> Result<()> {
        let l = CslStringList::from_iter(["ONE=1", "TWO=2"]);
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

    #[test]
    fn parse() -> Result<()> {
        let f = fixture()?;
        let s = f.to_string();
        let r: CslStringList = s.parse()?;

        assert_eq!(f.len(), r.len());
        assert_eq!(r.find_string("SOME_FLAG"), Some(3));
        assert_eq!(f.partial_find_string("THREE="), Some(2));
        assert_eq!(s, r.to_string());

        Ok(())
    }

    #[test]
    fn extend() -> Result<()> {
        let mut f = fixture()?;
        let o: CslStringList = "A=a B=b C=c D=d".parse()?;
        f.extend(&o);
        assert_eq!(f.len(), 8);
        assert_eq!(f.fetch_name_value("A"), Some("a".into()));
        Ok(())
    }
}
