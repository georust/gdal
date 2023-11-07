use std::path::{Path, PathBuf};
use crate::Dataset;

/// Enumeration of processing destination options.
///
/// Some GDAL operations require specification of a file or providing of an initialized dataset,
/// while others enable either. This type exists to handle the "either" case.
#[derive(Debug, Clone)]
pub enum Destination<'a> {
    File(PathBuf),
    Dataset(&'a Dataset),
}

impl<'a> Destination<'a> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self::File(path.as_ref().to_path_buf())
    }
    pub fn from_dataset(ds: &'a Dataset) -> Self {
        Self::Dataset(ds)
    }
}

impl<'a> From<&'a Dataset> for Destination<'a> {
    fn from(value: &'a Dataset) -> Self {
        Self::from_dataset(value)
    }
}

impl<P: AsRef<Path>> From<P> for Destination<'_> {
    fn from(value: P) -> Self {
        Self::from_path(value)
    }
}
