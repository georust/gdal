use crate::errors::{GdalError, Result};
use crate::Dataset;
use gdal_sys::OGRErr;
use std::ops::{Deref, DerefMut};

/// Represents an in-flight transaction on a dataset.
///
/// It can either be committed by calling [`commit`](Transaction::commit) or rolled back by calling
/// [`rollback`](Transaction::rollback).
///
/// If the transaction is not explicitly committed when it is dropped, it is implicitly rolled
/// back.
///
/// The transaction holds a mutable borrow on the `Dataset` that it was created from, so during the
/// lifetime of the transaction you will need to access the dataset by dereferencing the
/// `Transaction` through its [`Deref`] or [`DerefMut`] implementations.
#[derive(Debug)]
pub struct Transaction<'a> {
    dataset: &'a mut Dataset,
    rollback_on_drop: bool,
}

impl<'a> Transaction<'a> {
    fn new(dataset: &'a mut Dataset) -> Self {
        Transaction {
            dataset,
            rollback_on_drop: true,
        }
    }

    /// Returns a reference to the dataset from which this `Transaction` was created.
    #[deprecated = "Transaction now implements Deref<Target = Dataset>, so you can call Dataset methods on it directly. Use .deref() if you need a reference to the underlying Dataset."]
    pub fn dataset(&self) -> &Dataset {
        self.dataset
    }

    /// Returns a mutable reference to the dataset from which this `Transaction` was created.
    #[deprecated = "Transaction now implements DerefMut<Target = Dataset>, so you can call Dataset methods on it directly. Use .deref_mut() if you need a mutable reference to the underlying Dataset."]
    pub fn dataset_mut(&mut self) -> &mut Dataset {
        self.dataset
    }

    /// Commits this transaction.
    ///
    /// If the commit fails, will return [`OGRErr::OGRERR_FAILURE`].
    ///
    /// Depending on drivers, this may or may not abort layer sequential readings that are active.
    pub fn commit(mut self) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALDatasetCommitTransaction(self.dataset.c_dataset()) };
        self.rollback_on_drop = false;
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "GDALDatasetCommitTransaction",
            });
        }
        Ok(())
    }

    /// Rolls back the dataset to its state before the start of this transaction.
    ///
    /// If the rollback fails, will return [`OGRErr::OGRERR_FAILURE`].
    pub fn rollback(mut self) -> Result<()> {
        let rv = unsafe { gdal_sys::GDALDatasetRollbackTransaction(self.dataset.c_dataset()) };
        self.rollback_on_drop = false;
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "GDALDatasetRollbackTransaction",
            });
        }
        Ok(())
    }
}

impl Deref for Transaction<'_> {
    type Target = Dataset;

    fn deref(&self) -> &Self::Target {
        self.dataset
    }
}

impl DerefMut for Transaction<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dataset
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if self.rollback_on_drop {
            // We silently swallow any errors, because we have no way to report them from a drop
            // function apart from panicking.
            unsafe { gdal_sys::GDALDatasetRollbackTransaction(self.dataset.c_dataset()) };
        }
    }
}

impl Dataset {
    /// For datasources which support transactions, this creates a transaction.
    ///
    /// Because the transaction implements `DerefMut`, it can be used in place of the original
    /// `Dataset` to make modifications. All changes done after the start of the transaction are
    /// applied to the datasource when [`commit`](Transaction::commit) is called. They may be
    /// canceled by calling [`rollback`](Transaction::rollback) instead, or by dropping the
    /// `Transaction` without calling `commit`.
    ///
    /// Depending on the driver, using a transaction can give a huge performance improvement when
    /// creating a lot of geometry at once. This is because the driver doesn't need to commit every
    /// feature to disk individually.
    ///
    /// If starting the transaction fails, this function will return [`OGRErr::OGRERR_FAILURE`].
    /// For datasources that do not support transactions, this function will always return
    /// [`OGRErr::OGRERR_UNSUPPORTED_OPERATION`].
    ///
    /// Limitations:
    ///
    /// * Datasources which do not support efficient transactions natively may use less efficient
    ///   emulation of transactions instead; as of GDAL 3.1, this only applies to the closed-source
    ///   FileGDB driver, which (unlike OpenFileGDB) is not available in a GDAL build by default.
    ///
    /// * At the time of writing, transactions only apply on vector layers.
    ///
    /// * Nested transactions are not supported.
    ///
    /// * If an error occurs after a successful `start_transaction`, the whole transaction may or
    ///   may not be implicitly canceled, depending on the driver. For example, the PG driver will
    ///   cancel it, but the SQLite and GPKG drivers will not.
    ///
    /// Example:
    ///
    /// ```
    /// # use gdal::{Dataset };
    /// # use gdal::vector::LayerAccess;
    /// use gdal::vector::LayerOptions;
    /// #
    /// fn create_point_grid(dataset: &mut Dataset) -> gdal::errors::Result<()> {
    ///     use gdal::vector::Geometry;
    ///
    ///     // Start the transaction.
    ///     let mut txn = dataset.start_transaction()?;
    ///
    ///     let mut layer = txn.create_layer(LayerOptions {
    ///         name: "grid",
    ///         ty: gdal_sys::OGRwkbGeometryType::wkbPoint,
    ///         ..Default::default()
    ///     })?;
    ///     for y in 0..100 {
    ///         for x in 0..100 {
    ///             let wkt = format!("POINT ({} {})", x, y);
    ///             layer.create_feature(Geometry::from_wkt(&wkt)?)?;
    ///         }
    ///     }
    ///
    ///     // We got through without errors. Commit the transaction and return.
    ///     txn.commit()?;
    ///     Ok(())
    /// }
    /// #
    /// # fn main() -> gdal::errors::Result<()> {
    /// #     let driver = gdal::DriverManager::get_driver_by_name("SQLite")?;
    /// #     let mut dataset = driver.create_vector_only(":memory:")?;
    /// #     create_point_grid(&mut dataset)?;
    /// #     assert_eq!(dataset.layer(0)?.features().count(), 10000);
    /// #     Ok(())
    /// # }
    /// ```
    pub fn start_transaction(&mut self) -> Result<Transaction<'_>> {
        let force = 1;
        let rv = unsafe { gdal_sys::GDALDatasetStartTransaction(self.c_dataset(), force) };
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "GDALDatasetStartTransaction",
            });
        }
        Ok(Transaction::new(self))
    }
}

/// Represents maybe a transaction on a dataset for speed reasons.
///
/// It can be committed by calling [`commit`](MaybeTransaction::commit), but unlike [`Transaction`] it can not be rolled back. The transaction part is to make the process speedy when possible. If the [`Dataset`] does not support transaction, it does nothing.
///
/// If the transaction is not explicitly committed when it is dropped, it is implicitly committed, but you will not know if it fails.
///
/// The transaction holds a mutable borrow on the `Dataset` that it was created from, so during the
/// lifetime of the transaction you will need to access the dataset by dereferencing the
/// `MaybeTransaction` through its [`Deref`] or [`DerefMut`] implementations.
#[derive(Debug)]
pub struct MaybeTransaction<'a> {
    dataset: &'a mut Dataset,
    is_transaction: bool,
    commit_on_drop: bool,
}

impl<'a> MaybeTransaction<'a> {
    fn new(dataset: &'a mut Dataset, is_transaction: bool) -> Self {
        MaybeTransaction {
            dataset,
            is_transaction,
            commit_on_drop: true,
        }
    }

    pub fn is_transaction(&self) -> bool {
        self.is_transaction
    }

    /// Commits this transaction.
    ///
    /// If the commit fails, will return [`OGRErr::OGRERR_FAILURE`].
    ///
    /// Depending on drivers, this may or may not abort layer sequential readings that are active.
    pub fn commit(mut self) -> Result<()> {
        if !self.is_transaction {
            return Ok(());
        }

        let rv = unsafe { gdal_sys::GDALDatasetCommitTransaction(self.dataset.c_dataset()) };
        self.commit_on_drop = false;
        if rv != OGRErr::OGRERR_NONE {
            return Err(GdalError::OgrError {
                err: rv,
                method_name: "GDALDatasetCommitTransaction",
            });
        }
        Ok(())
    }
}

impl<'a> Deref for MaybeTransaction<'a> {
    type Target = Dataset;

    fn deref(&self) -> &Self::Target {
        self.dataset
    }
}

impl<'a> DerefMut for MaybeTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dataset
    }
}

impl Dataset {
    pub fn maybe_start_transaction(&mut self) -> MaybeTransaction<'_> {
        let force = 1;
        let rv = unsafe { gdal_sys::GDALDatasetStartTransaction(self.c_dataset(), force) };

        MaybeTransaction::new(self, rv == OGRErr::OGRERR_NONE)
    }
}

impl<'a> Drop for MaybeTransaction<'a> {
    fn drop(&mut self) {
        if self.commit_on_drop {
            // We silently swallow any errors, because we have no way to report them from a drop
            // function apart from panicking.
            unsafe { gdal_sys::GDALDatasetCommitTransaction(self.dataset.c_dataset()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{fixture, open_dataset_for_update, open_gpkg_for_update};
    use crate::vector::{Geometry, LayerAccess};
    use crate::Dataset;

    fn polygon() -> Geometry {
        Geometry::from_wkt("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))").unwrap()
    }

    #[test]
    fn test_start_transaction() {
        let (_temp_path, mut ds) = open_gpkg_for_update(&fixture("poly.gpkg"));
        let txn = ds.start_transaction();
        assert!(txn.is_ok());
    }

    #[test]
    fn test_transaction_commit() {
        let (_temp_path, mut ds) = open_gpkg_for_update(&fixture("poly.gpkg"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        let txn = ds.start_transaction().unwrap();
        let mut layer = txn.layer(0).unwrap();
        layer.create_feature(polygon()).unwrap();
        assert!(txn.commit().is_ok());

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count + 1);
    }

    #[test]
    fn test_transaction_rollback() {
        let (_temp_path, mut ds) = open_gpkg_for_update(&fixture("poly.gpkg"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        let txn = ds.start_transaction().unwrap();
        let mut layer = txn.layer(0).unwrap();
        layer.create_feature(polygon()).unwrap();
        assert!(txn.rollback().is_ok());

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count);
    }

    #[test]
    fn test_transaction_implicit_rollback() {
        let (_temp_path, mut ds) = open_gpkg_for_update(&fixture("poly.gpkg"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        {
            let txn = ds.start_transaction().unwrap();
            let mut layer = txn.layer(0).unwrap();
            layer.create_feature(polygon()).unwrap();
        } // txn is dropped here.

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count);
    }

    #[test]
    fn test_start_transaction_unsupported() {
        let mut ds = Dataset::open(fixture("roads.geojson")).unwrap();
        assert!(ds.start_transaction().is_err());
    }

    #[test]
    fn test_maybe_start_transaction() {
        let (_temp_path, mut ds) = open_gpkg_for_update(&fixture("poly.gpkg"));
        let txn = ds.maybe_start_transaction();
        assert!(txn.is_transaction());
        let mut ds = Dataset::open(fixture("roads.geojson")).unwrap();
        let txn = ds.maybe_start_transaction();
        assert!(!txn.is_transaction());
    }

    #[test]
    fn test_maybe_transaction_commit() {
        let (_temp_path, mut ds) = open_gpkg_for_update(&fixture("poly.gpkg"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        let txn = ds.maybe_start_transaction();
        let mut layer = txn.layer(0).unwrap();
        layer.create_feature(polygon()).unwrap();
        assert!(txn.commit().is_ok());

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count + 1);
    }

    #[test]
    fn test_maybe_transaction_commit_unsupported() {
        let (_temp_path, mut ds) = open_dataset_for_update(&fixture("roads.geojson"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        let txn = ds.maybe_start_transaction();
        let mut layer = txn.layer(0).unwrap();
        layer.create_feature(polygon()).unwrap();
        assert!(txn.commit().is_ok());

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count + 1);
    }

    #[test]
    fn test_maybe_transaction_implicit_commit() {
        let (_temp_path, mut ds) = open_gpkg_for_update(&fixture("poly.gpkg"));
        let orig_feature_count = ds.layer(0).unwrap().feature_count();

        {
            let txn = ds.maybe_start_transaction();
            let mut layer = txn.layer(0).unwrap();
            layer.create_feature(polygon()).unwrap();
        } // txn is dropped here.

        assert_eq!(ds.layer(0).unwrap().feature_count(), orig_feature_count + 1);
    }
}
