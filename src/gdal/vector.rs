use std::ptr::null;
use std::str::raw;
use libc::{c_int, c_char};
use sync::mutex::{StaticMutex, MUTEX_INIT};


#[link(name="gdal")]
extern {
    fn OGRRegisterAll();
    fn OGROpen(pszName: *c_char, bUpdate: c_int, pahDriverList: *()) -> *();
    fn OGR_DS_GetLayerCount(hDS: *()) -> c_int;
    fn OGR_DS_Destroy(hDataSource: *());
    fn OGR_DS_GetLayer(hDS: *(), iLayer: c_int) -> *();
    fn OGR_L_GetNextFeature(hLayer: *()) -> *();
    fn OGR_F_GetFieldIndex(hFeat: *(), pszName: *c_char) -> c_int;
    fn OGR_F_GetFieldAsString(hFeat: *(), iField: c_int) -> *c_char;
    fn OGR_F_Destroy(hFeat: *());
}


static mut LOCK: StaticMutex = MUTEX_INIT;
static mut registered_drivers: bool = false;

fn register_drivers() {
    unsafe {
        let _g = LOCK.lock();
        if ! registered_drivers {
            OGRRegisterAll();
            registered_drivers = true;
        }
    }
}


pub struct VectorDataset {
    c_dataset: *(),
}


impl VectorDataset {
    pub fn get_layer_count(&self) -> int {
        return unsafe { OGR_DS_GetLayerCount(self.c_dataset) } as int;
    }

    pub fn get_layer<'a>(&'a self, idx: int) -> Option<Layer<'a>> {
        let c_layer = unsafe { OGR_DS_GetLayer(self.c_dataset, idx as c_int) };
        return match c_layer.is_null() {
            true  => None,
            false => Some(Layer{vector_dataset: self, c_layer: c_layer}),
        };
    }
}


impl Drop for VectorDataset {
    fn drop(&mut self) {
        unsafe { OGR_DS_Destroy(self.c_dataset); }
    }
}


pub struct Layer<'a> {
    vector_dataset: &'a VectorDataset,
    c_layer: *(),
}


impl<'a> Layer<'a> {
    pub fn features<'a>(&'a self) -> FeatureIterator<'a> {
        return FeatureIterator{layer: self};
    }
}


pub struct FeatureIterator<'a> {
    layer: &'a Layer<'a>,
}


impl<'a> Iterator<Feature<'a>> for FeatureIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<Feature<'a>> {
        let c_feature = unsafe { OGR_L_GetNextFeature(self.layer.c_layer) };
        return match c_feature.is_null() {
            true  => None,
            false => Some(Feature{layer: self.layer, c_feature: c_feature}),
        };
    }
}


pub struct Feature<'a> {
    layer: &'a Layer<'a>,
    c_feature: *(),
}


impl<'a> Feature<'a> {
    pub fn get_field(&self, name: String) -> String {
        return name.with_c_str(|c_name| {
            unsafe {
                let field_id = OGR_F_GetFieldIndex(self.c_feature, c_name);
                let c_value = OGR_F_GetFieldAsString(self.c_feature, field_id);
                return raw::from_c_str(c_value);
            }
        });
    }
}


#[unsafe_destructor]
impl<'a> Drop for Feature<'a> {
    fn drop(&mut self) {
        unsafe { OGR_F_Destroy(self.c_feature); }
    }
}


pub fn open(path: &Path) -> Option<VectorDataset> {
    register_drivers();
    let filename = path.as_str().unwrap();
    let c_dataset = filename.with_c_str(|c_filename| {
        return unsafe { OGROpen(c_filename, 0, null()) };
    });
    return match c_dataset.is_null() {
        true  => None,
        false => Some(VectorDataset{c_dataset: c_dataset}),
    };
}



#[cfg(test)]
mod test {
    use std::os::getenv;
    use std::path::Path;
    use super::{Feature, open};


    fn fixture_path(name: &str) -> Path {
        let envvar = "RUST_GDAL_TEST_FIXTURES";
        let fixtures = match getenv(envvar) {
            Some(p) => Path::new(p),
            None => fail!("Environment variable {} not set", envvar)
        };
        let rv = fixtures.join(name);
        return rv;
    }


    #[test]
    fn test_layer_count() {
        let ds = open(&fixture_path("roads.geojson")).unwrap();
        assert_eq!(ds.get_layer_count(), 1);
    }


    #[test]
    fn test_iterate_features() {
        let ds = open(&fixture_path("roads.geojson")).unwrap();
        let layer = ds.get_layer(0).unwrap();
        let features: Vec<Feature> = layer.features().collect();
        assert_eq!(features.len(), 21);
    }


    #[test]
    fn test_get_field() {
        let ds = open(&fixture_path("roads.geojson")).unwrap();
        let layer = ds.get_layer(0).unwrap();
        let feature = layer.features().next().unwrap();
        assert_eq!(feature.get_field("highway".to_string()),
                   "footway".to_string());
        assert_eq!(
            layer.features()
                 .count(|f|
                    f.get_field("highway".to_string()) ==
                    "residential".to_string()),
            2);
    }
}
