use std::ffi::{CStr, CString};
use std::ops::{Deref, DerefMut};

use crate::Dataset;
use gdal_sys::{CPLErr, GDALDatasetH, OGRGeometryH};

use crate::errors::*;
use crate::utils::_last_cpl_err;
use crate::vector::{sql, Geometry, Layer, LayerAccess};

/// The result of a SQL query executed by
/// [`Dataset::execute_sql()`](Dataset::execute_sql()). It is just a thin wrapper around a
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

/// [Dataset] methods relating to SQL over vector datasets.
impl Dataset {
    /// Execute a SQL query against the Dataset. It is equivalent to calling
    /// [`GDALDatasetExecuteSQL`](https://gdal.org/api/raster_c_api.html#_CPPv421GDALDatasetExecuteSQL12GDALDatasetHPKc12OGRGeometryHPKc).
    /// Returns a [`ResultSet`], which can be treated just as any other [`Layer`].
    ///
    /// Queries such as `ALTER TABLE`, `CREATE INDEX`, etc. have no [`ResultSet`], and return
    /// `None`, which is distinct from an empty [`ResultSet`].
    ///
    /// # Arguments
    /// * `query`: The SQL query
    /// * `spatial_filter`: Limit results of the query to features that intersect the given
    ///   [`Geometry`]
    /// * `dialect`: The dialect of SQL to use. See
    ///   <https://gdal.org/user/ogr_sql_sqlite_dialect.html>
    ///
    /// # Example
    ///
    /// ```
    /// # use gdal::Dataset;
    /// # use std::path::Path;
    /// use gdal::vector::sql;
    /// use gdal::vector::LayerAccess;
    ///
    /// let ds = Dataset::open(Path::new("fixtures/roads.geojson")).unwrap();
    /// let query = "SELECT kind, is_bridge, highway FROM roads WHERE highway = 'pedestrian'";
    /// let mut result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT).unwrap().unwrap();
    ///
    /// assert_eq!(10, result_set.feature_count());
    ///
    /// for feature in result_set.features() {
    ///     let highway = feature
    ///         .field("highway")
    ///         .unwrap()
    ///         .unwrap()
    ///         .into_string()
    ///         .unwrap();
    ///
    ///     assert_eq!("pedestrian", highway);
    /// }
    /// ```
    pub fn execute_sql<S: AsRef<str>>(
        &self,
        query: S,
        spatial_filter: Option<&Geometry>,
        dialect: Dialect,
    ) -> Result<Option<ResultSet>> {
        let query = CString::new(query.as_ref())?;

        let dialect_c_str = match dialect {
            Dialect::DEFAULT => None,
            Dialect::OGR => Some(unsafe { CStr::from_bytes_with_nul_unchecked(OGRSQL) }),
            Dialect::SQLITE => Some(unsafe { CStr::from_bytes_with_nul_unchecked(SQLITE) }),
        };

        self._execute_sql(query, spatial_filter, dialect_c_str)
    }

    fn _execute_sql(
        &self,
        query: CString,
        spatial_filter: Option<&Geometry>,
        dialect_c_str: Option<&CStr>,
    ) -> Result<Option<sql::ResultSet>> {
        let mut filter_geom: OGRGeometryH = std::ptr::null_mut();

        let dialect_ptr = match dialect_c_str {
            None => std::ptr::null(),
            Some(d) => d.as_ptr(),
        };

        if let Some(spatial_filter) = spatial_filter {
            filter_geom = unsafe { spatial_filter.c_geometry() };
        }

        let c_dataset = self.c_dataset();

        unsafe { gdal_sys::CPLErrorReset() };

        let c_layer = unsafe {
            gdal_sys::GDALDatasetExecuteSQL(c_dataset, query.as_ptr(), filter_geom, dialect_ptr)
        };

        let cpl_err = unsafe { gdal_sys::CPLGetLastErrorType() };

        if cpl_err != CPLErr::CE_None {
            return Err(_last_cpl_err(cpl_err));
        }

        if c_layer.is_null() {
            return Ok(None);
        }

        let layer = unsafe { Layer::from_c_layer(self, c_layer) };

        Ok(Some(sql::ResultSet {
            layer,
            dataset: c_dataset,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::test_utils::SuppressGDALErrorLog;
    use crate::{
        test_utils::fixture,
        vector::{sql, Geometry, LayerAccess},
        Dataset,
    };

    #[test]
    fn test_sql() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let query = "SELECT kind, is_bridge, highway FROM roads WHERE highway = 'pedestrian'";
        let mut result_set = ds
            .execute_sql(query, None, sql::Dialect::DEFAULT)
            .unwrap()
            .unwrap();

        let field_names: HashSet<_> = result_set
            .defn()
            .fields()
            .map(|field| field.name())
            .collect();

        let mut correct_field_names = HashSet::new();
        correct_field_names.insert("kind".into());
        correct_field_names.insert("is_bridge".into());
        correct_field_names.insert("highway".into());

        assert_eq!(correct_field_names, field_names);
        assert_eq!(10, result_set.feature_count());

        for feature in result_set.features() {
            let highway = feature
                .field("highway")
                .unwrap()
                .unwrap()
                .into_string()
                .unwrap();

            assert_eq!("pedestrian", highway);
        }
    }

    #[test]
    fn test_sql_with_spatial_filter() {
        let query = "SELECT * FROM roads WHERE highway = 'pedestrian'";
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let bbox = Geometry::bbox(26.1017, 44.4297, 26.1025, 44.4303).unwrap();
        let mut result_set = ds
            .execute_sql(query, Some(&bbox), sql::Dialect::DEFAULT)
            .unwrap()
            .unwrap();

        assert_eq!(2, result_set.feature_count());
        let mut correct_fids = HashSet::new();
        correct_fids.insert(252725993);
        correct_fids.insert(23489656);

        let mut fids = HashSet::new();
        for feature in result_set.features() {
            let highway = feature
                .field("highway")
                .unwrap()
                .unwrap()
                .into_string()
                .unwrap();

            assert_eq!("pedestrian", highway);
            fids.insert(feature.fid().unwrap());
        }

        assert_eq!(correct_fids, fids);
    }

    #[test]
    fn test_sql_with_dialect() {
        let query = "SELECT * FROM roads WHERE highway = 'pedestrian' and NumPoints(GEOMETRY) = 3";
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let bbox = Geometry::bbox(26.1017, 44.4297, 26.1025, 44.4303).unwrap();
        let mut result_set = ds
            .execute_sql(query, Some(&bbox), sql::Dialect::SQLITE)
            .unwrap()
            .unwrap();

        assert_eq!(1, result_set.feature_count());
        let mut features: Vec<_> = result_set.features().collect();
        let feature = features.pop().unwrap();
        let highway = feature
            .field("highway")
            .unwrap()
            .unwrap()
            .into_string()
            .unwrap();

        assert_eq!("pedestrian", highway);
    }

    #[test]
    fn test_sql_empty_result() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let query = "SELECT kind, is_bridge, highway FROM roads WHERE highway = 'jazz hands 👐'";
        let mut result_set = ds
            .execute_sql(query, None, sql::Dialect::DEFAULT)
            .unwrap()
            .unwrap();
        assert_eq!(0, result_set.feature_count());
        assert_eq!(0, result_set.features().count());
    }

    #[test]
    fn test_sql_no_result() {
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let query = "ALTER TABLE roads ADD COLUMN fun integer";
        let result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT).unwrap();
        assert!(result_set.is_none());
    }

    #[test]
    fn test_sql_bad_query() {
        let _nolog = SuppressGDALErrorLog::new();
        let ds = Dataset::open(fixture("roads.geojson")).unwrap();

        let query = "SELECT nope FROM roads";
        let result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT);
        assert!(result_set.is_err());

        let query = "SELECT nope FROM";
        let result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT);
        assert!(result_set.is_err());

        let query = "SELECT ninetynineredballoons(highway) FROM roads";
        let result_set = ds.execute_sql(query, None, sql::Dialect::DEFAULT);
        assert!(result_set.is_err());
    }
}
