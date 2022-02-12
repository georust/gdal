use std::ops::{Deref, DerefMut};

use gdal_sys::GDALDatasetH;

use crate::vector::{Layer, LayerAccess};

/// The result of a SQL query executed by
/// [`Dataset::execute_sql()`](crate::Dataset::execute_sql()). It is just a thin wrapper around a
/// [`Layer`], and you can treat it as such.
#[derive(Debug)]
pub struct ResultSet<'a> {
    pub(crate) layer: Layer<'a>,
    pub(crate) dataset: GDALDatasetH,
}

impl<'a> Deref for ResultSet<'a> {
    type Target = Layer<'a>;

    fn deref(&self) -> &Self::Target {
        &self.layer
    }
}

impl<'a> DerefMut for ResultSet<'a> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.layer
    }
}

impl<'a> Drop for ResultSet<'a> {
    fn drop(&mut self) {
        unsafe { gdal_sys::GDALDatasetReleaseResultSet(self.dataset, self.layer.c_layer()) };
    }
}

/// Represents valid SQL dialects to use in SQL queries. See
/// <https://gdal.org/user/ogr_sql_sqlite_dialect.html>
#[allow(clippy::upper_case_acronyms)]
pub enum Dialect {
    /// Use the default dialect. This is OGR SQL unless the underlying driver has a native dialect,
    /// such as MySQL, Postgres, Oracle, etc.
    DEFAULT,

    /// Explicitly choose OGR SQL regardless of if the underlying driver has a native dialect.
    OGR,

    /// SQLite dialect. If the data set is not actually a SQLite database, then a virtual SQLite
    /// table is created to execute the query.
    SQLITE,
}

pub(crate) const OGRSQL: &[u8] = b"OGRSQL\0";
pub(crate) const SQLITE: &[u8] = b"SQLITE\0";
