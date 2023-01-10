use crate::errors::*;
use crate::gdal_major_object::MajorObject;
use crate::utils::{_last_cpl_err, _last_null_pointer_err, _string, _string_array};
use gdal_sys::{self, CPLErr};
use std::ffi::CString;

/// General-Purpose Metadata API
///
/// The [`Metadata`] trait exposes a simple, general-purpose metadata model for various types
/// in the GDAL datamodel, including [`Driver`](crate::Driver),
/// [`Dataset`](crate::Dataset), [`RasterBand`](crate::raster::RasterBand),
/// and [`Layer`](crate::vector::Layer).
/// These data are comprised of key-value strings, organized under parent keys called "domains".
/// This includes the empty-string (`""`) root domain. There's even an `xml:` domain with it's own
/// world of content.
///
/// GDAL's metadata structures could be seen as peculiar, and there are driver-specific nuances to navigate,
/// but in scientifically rich datasets, or projects aiming for archival-quality content, its capabilities
/// can fulfill sophisticated requirements.
///
/// Reading the _Metadata_ section in the [GDAL Raster Data Model](https://gdal.org/user/raster_data_model.html#metadata)
/// document can help if you need to deep, fine-grained metadata access, or examples on how it is used.
///
/// # Example
///
/// ```rust, no_run
/// use gdal::{Dataset, Metadata};
/// # fn main() -> gdal::errors::Result<()> {
/// let dataset = Dataset::open("fixtures/tinymarble.tif")?;
/// // `description` on a `Dataset` is usually the file name.
/// let desc = dataset.description()?;
/// dbg!(desc);
///
/// // `IMAGE_STRUCTURE` is one of the standard domains
/// let md_domain = "IMAGE_STRUCTURE";
/// // The `INTERLEAVE` key provides a hint as to how pixel data is organized
/// let md_key = "INTERLEAVE";
/// // NB: `domain` comes **after** the `key`
/// let interleave = dataset.metadata_item(&md_key, &md_domain);
/// dbg!(interleave);
/// # Ok(())
/// # }
/// ```
pub trait Metadata: MajorObject {
    /// For most [`crate::Dataset`]s, method returns this is the originating filename.
    /// For [`crate::raster::RasterBand`]s it is a description (if supported) or `""`.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::{Dataset, Metadata};
    /// # fn main() -> gdal::errors::Result<()> {
    /// let dataset = Dataset::open("fixtures/tinymarble.tif")?;
    /// // `description` on a `Dataset` is usually the file name.
    /// let desc = dataset.description()?;
    /// assert_eq!(desc, "fixtures/tinymarble.tif");
    /// # Ok(())
    /// # }
    /// ```
    fn description(&self) -> Result<String> {
        let c_res = unsafe { gdal_sys::GDALGetDescription(self.gdal_object_ptr()) };
        if c_res.is_null() {
            return Err(_last_null_pointer_err("GDALGetDescription"));
        }
        Ok(_string(c_res))
    }

    /// Metadata in GDAL is partitioned into namespaces, knows as "domains" in the
    /// [GDAL Data Model](https://gdal.org/user/raster_data_model.html#metadata).
    /// GDAL types with metadata (a.k.a. "Major Objects") have a default or "root" domain
    /// identified by the empty string (`""`).
    /// Specific "Major Object" types may have other conventionally recognized domains.
    /// For example, in raster [`Dataset`][crate::Dataset]s you may come across the domains
    /// `SUBDATASETS`, `IMAGE_STRUCTURE`, `RPC`, `IMAGERY`, `xml:`, etc.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::{Dataset, Metadata};
    /// # fn main() -> gdal::errors::Result<()> {
    /// let dataset = Dataset::open("fixtures/labels.tif")?;
    /// assert!(dataset.metadata_domains().contains(&"IMAGE_STRUCTURE".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    fn metadata_domains(&self) -> Vec<String> {
        let mut domains = Vec::new();
        let c_res = unsafe { gdal_sys::GDALGetMetadataDomainList(self.gdal_object_ptr()) };

        if !c_res.is_null() {
            domains.append(&mut _string_array(c_res));
        }
        unsafe { gdal_sys::CSLDestroy(c_res) };

        domains
    }

    /// Get all the metadata values within the given `domain`. Returns `None` if
    /// domain is not defined.
    /// Entries in the returned `Vec<String>` are formatted as "Name=value" pairs
    ///
    /// # Arguments
    ///
    /// * domain – the domain of interest. Use `""` for the default domain.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::{Dataset, Metadata};
    /// # fn main() -> gdal::errors::Result<()> {
    /// let dataset = Dataset::open("fixtures/labels.tif")?;
    /// assert_eq!(dataset.metadata_domain("IMAGE_STRUCTURE").unwrap(),vec!["INTERLEAVE=BAND"] );
    /// # Ok(())
    /// # }
    /// ```
    fn metadata_domain(&self, domain: &str) -> Option<Vec<String>> {
        let mut metadata = Vec::new();
        if let Ok(c_domain) = CString::new(domain.to_owned()) {
            let c_res =
                unsafe { gdal_sys::GDALGetMetadata(self.gdal_object_ptr(), c_domain.as_ptr()) };

            if c_res.is_null() {
                return None;
            } else {
                metadata.append(&mut _string_array(c_res));
            }
        }

        Some(metadata)
    }

    /// Get a single metadata entry, as indicated by `key` and `domain`.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::{Dataset, Metadata};
    /// # fn main() -> gdal::errors::Result<()> {
    /// let dataset = Dataset::open("fixtures/labels.tif")?;
    /// assert_eq!(dataset.metadata_item("INTERLEAVE", "IMAGE_STRUCTURE").unwrap(),"BAND");
    /// # Ok(())
    /// # }
    /// ```
    fn metadata_item(&self, key: &str, domain: &str) -> Option<String> {
        if let Ok(c_key) = CString::new(key.to_owned()) {
            if let Ok(c_domain) = CString::new(domain.to_owned()) {
                let c_res = unsafe {
                    gdal_sys::GDALGetMetadataItem(
                        self.gdal_object_ptr(),
                        c_key.as_ptr(),
                        c_domain.as_ptr(),
                    )
                };
                if !c_res.is_null() {
                    return Some(_string(c_res));
                }
            }
        }
        None
    }

    /// Set a metadata item in given `domain` at given `key`.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::{DriverManager, Metadata};
    /// # fn main() -> gdal::errors::Result<()> {
    /// let mut driver = DriverManager::get_driver_by_name("MEM")?;
    /// let (domain, key, value) = ("FOOBAR", "fake", "data");
    /// assert!(driver.metadata_domain(domain).is_none());
    /// driver.set_metadata_item(key, value, domain)?;
    /// assert!(driver.metadata_domain(domain).is_some());
    /// assert_eq!(driver.metadata_item(key, domain), Some(value.to_string()));
    /// # Ok(())
    /// # }
    /// ```
    fn set_metadata_item(&mut self, key: &str, value: &str, domain: &str) -> Result<()> {
        let c_key = CString::new(key)?;
        let c_domain = CString::new(domain)?;
        let c_value = CString::new(value)?;

        let c_res = unsafe {
            gdal_sys::GDALSetMetadataItem(
                self.gdal_object_ptr(),
                c_key.as_ptr(),
                c_value.as_ptr(),
                c_domain.as_ptr(),
            )
        };
        if c_res != CPLErr::CE_None {
            return Err(_last_cpl_err(c_res));
        }
        Ok(())
    }

    /// For Datasets this sets the dataset name; normally
    /// application code should not set the "description" for
    /// GDALDatasets. For RasterBands it is actually a description
    /// (if supported) or `""`.
    fn set_description(&mut self, description: &str) -> Result<()> {
        let c_description = CString::new(description.to_owned())?;
        unsafe { gdal_sys::GDALSetDescription(self.gdal_object_ptr(), c_description.as_ptr()) };
        Ok(())
    }

    /// Get an iterator over metadata entries, across all domains.
    ///
    /// # Example
    ///
    /// ```rust, no_run
    /// use gdal::{Dataset, Metadata, MetadataEntry};
    /// # fn main() -> gdal::errors::Result<()> {
    /// let dataset = Dataset::open("fixtures/tinymarble.tif")?;
    /// for MetadataEntry { domain, key, value } in dataset.metadata() {
    ///   let domain = if domain == "" { "DEFAULT".to_string() } else { domain };
    ///   println!("{domain}: {key}={value}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// Output:
    /// ```text
    /// IMAGE_STRUCTURE: INTERLEAVE=PIXEL
    /// DEFAULT: AREA_OR_POINT=Area
    /// ...
    /// DERIVED_SUBDATASETS: DERIVED_SUBDATASET_1_NAME=DERIVED_SUBDATASET:LOGAMPLITUDE:fixtures/tinymarble.tif
    /// DERIVED_SUBDATASETS: DERIVED_SUBDATASET_1_DESC=log10 of amplitude of input bands from fixtures/tinymarble.tif
    /// ```
    fn metadata(&self) -> MetadataIter
    where
        Self: Sized,
    {
        MetadataIter::new(self)
    }
}

/// Standalone metadata entry, as returned by iterator from [`Metadata::metadata`].
///
/// Defined by it's parent `domain`, and `key`/`value` pair.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MetadataEntry {
    pub domain: String,
    pub key: String,
    pub value: String,
}

impl MetadataEntry {
    pub fn new<D, K, V>(domain: D, key: K, value: V) -> Self
    where
        D: Into<String>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            domain: domain.into(),
            key: key.into(),
            value: value.into(),
        }
    }

    /// Determine if this entry is from the default domain, which is named `""`.
    pub fn is_default_domain(&self) -> bool {
        self.domain.is_empty()
    }
}

/// Metadata iterator state
pub struct MetadataIter<'a> {
    stream: Box<dyn Iterator<Item = MetadataEntry> + 'a>,
}

/// Iterator over metadata entries
impl<'a> MetadataIter<'a> {
    fn new<P: Metadata>(parent: &'a P) -> Self {
        let stream = parent
            .metadata_domains()
            .into_iter()
            .flat_map(move |domain| {
                let keyvals = parent.metadata_domain(&domain).unwrap_or_default();
                keyvals.into_iter().filter_map(move |keyval| {
                    keyval
                        .split_once('=')
                        .map(|(key, value)| MetadataEntry::new(domain.clone(), key, value))
                })
            });

        Self {
            stream: Box::new(stream),
        }
    }
}

impl<'a> Iterator for MetadataIter<'a> {
    type Item = MetadataEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::metadata::MetadataEntry;
    use crate::test_utils::fixture;
    use crate::*;

    #[test]
    fn test_get_dataset_driver() {
        let dataset = Dataset::open(fixture("tinymarble.png")).unwrap();
        let driver = dataset.driver();
        assert_eq!(driver.short_name(), "PNG");
        assert_eq!(driver.long_name(), "Portable Network Graphics");
    }

    #[test]
    fn test_get_description() {
        let driver = DriverManager::get_driver_by_name("mem").unwrap();
        assert_eq!(driver.description().unwrap(), "MEM".to_string());
    }

    #[test]
    fn test_get_metadata_domains() {
        let dataset = Dataset::open(fixture("tinymarble.png")).unwrap();
        let mut domains = dataset.metadata_domains();
        if domains[0].is_empty() {
            domains.remove(0);
        }

        assert_eq!(
            domains,
            vec!(
                "IMAGE_STRUCTURE",
                "xml:XMP",
                "DERIVED_SUBDATASETS",
                "COLOR_PROFILE"
            )
        );
    }

    #[test]
    fn test_get_metadata_domain() {
        let dataset = Dataset::open(fixture("tinymarble.png")).unwrap();
        let domain = "None";
        let meta = dataset.metadata_domain(domain);
        assert_eq!(meta, None);

        let domain = "IMAGE_STRUCTURE";
        let meta = dataset.metadata_domain(domain);
        assert_eq!(meta, Some(vec!(String::from("INTERLEAVE=PIXEL"))));
    }

    #[test]
    fn test_get_metadata_item() {
        let dataset = Dataset::open(fixture("tinymarble.png")).unwrap();
        let key = "None";
        let domain = "None";
        let meta = dataset.metadata_item(key, domain);
        assert_eq!(meta, None);

        let key = "INTERLEAVE";
        let domain = "IMAGE_STRUCTURE";
        let meta = dataset.metadata_item(key, domain);
        assert_eq!(meta, Some(String::from("PIXEL")));
    }

    #[test]
    fn test_set_metadata_item() {
        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let mut dataset = driver.create("", 1, 1, 1).unwrap();

        let key = "Test_Key";
        let domain = "Test_Domain";
        let value = "Test_Value";
        let result = dataset.set_metadata_item(key, value, domain);
        assert!(result.is_ok());

        let result = dataset.metadata_item(key, domain);
        assert_eq!(Some(value.to_owned()), result);
    }

    #[test]
    fn test_set_description() {
        let driver = DriverManager::get_driver_by_name("MEM").unwrap();
        let dataset = driver.create("", 1, 1, 1).unwrap();
        let mut band = dataset.rasterband(1).unwrap();

        let description = "A merry and cheerful band description";
        assert_eq!(band.description().unwrap(), "");

        band.set_description(description).unwrap();
        assert_eq!(band.description().unwrap(), description);
    }

    #[test]
    fn test_md_iter() {
        // Driver metadata...
        let driver = DriverManager::get_driver_by_name("GTiff").unwrap();
        driver.metadata().any(|e| e.key == "LIBGEOTIFF");

        // Dataset metadata...
        let ds = Dataset::open(fixture("m_3607824_se_17_1_20160620_sub.tif")).unwrap();
        assert_eq!(ds.metadata_item("AREA_OR_POINT", ""), Some("Area".into()));
        assert!(ds
            .metadata()
            .any(|e| e == MetadataEntry::new("", "AREA_OR_POINT", "Area")));
        assert!(ds.metadata().any(|e| e.domain == "DERIVED_SUBDATASETS"));

        // RasterBand metadata...
        let ds = Dataset::open(fixture("labels.tif")).unwrap();
        let band = ds.rasterband(1).unwrap();
        assert!(band.metadata().any(|e| e.key == "CLASSES"));
    }
}
