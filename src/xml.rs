use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};
use std::ptr::NonNull;
use std::str::FromStr;

use gdal_sys::{
    CPLCloneXMLTree, CPLDestroyXMLNode, CPLErr, CPLParseXMLString, CPLSerializeXMLTree, CPLXMLNode,
};

use crate::utils::_last_cpl_err;

/// An XML node, as captured from GDAL serialization APIs.
pub struct GdalXmlNode(NonNull<CPLXMLNode>);

impl GdalXmlNode {
    /// Create a Self from a raw pointer.
    ///
    /// # Safety
    /// Caller is responsible for ensuring `ptr` is not null, and
    /// ownership of `ptr` is properly transferred.
    pub unsafe fn from_ptr(ptr: *mut CPLXMLNode) -> GdalXmlNode {
        Self(NonNull::new_unchecked(ptr))
    }

    pub fn as_ptr(&self) -> *const CPLXMLNode {
        self.0.as_ptr()
    }

    pub fn as_ptr_mut(&self) -> *mut CPLXMLNode {
        self.0.as_ptr()
    }
}

impl Clone for GdalXmlNode {
    fn clone(&self) -> Self {
        unsafe { GdalXmlNode::from_ptr(CPLCloneXMLTree(self.as_ptr())) }
    }
}

impl Drop for GdalXmlNode {
    fn drop(&mut self) {
        unsafe { CPLDestroyXMLNode(self.0.as_mut()) };
    }
}

impl FromStr for GdalXmlNode {
    type Err = crate::errors::GdalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s2 = CString::new(s)?;
        let c_xml = unsafe { CPLParseXMLString(s2.as_ptr()) };
        if c_xml.is_null() {
            Err(_last_cpl_err(CPLErr::CE_Failure))
        } else {
            Ok(unsafe { GdalXmlNode::from_ptr(c_xml) })
        }
    }
}

impl Display for GdalXmlNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = unsafe { CString::from_raw(CPLSerializeXMLTree(self.as_ptr_mut())) };
        f.write_str(s.to_string_lossy().trim_end())
    }
}

impl Debug for GdalXmlNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = self.to_string();
        if !f.alternate() {
            // Flatten the display string to fit on one line.
            s = s.replace(['\n', '\r', ' ', '\t'], "");
        }
        f.write_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use crate::xml::GdalXmlNode;

    #[test]
    fn serde() {
        let src = r#"<foo bar="baz">yeet</foo>"#;
        let xml: GdalXmlNode = src.parse().unwrap();
        let s = xml.to_string();
        assert_eq!(src, s.trim());
    }
}
