use std::ptr::null;
use std::str::raw;
use libc::{c_int, c_char, c_double};
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
    fn OGR_F_GetFieldDefnRef(hFeat: *(), i: c_int) -> *();
    fn OGR_F_GetFieldAsString(hFeat: *(), iField: c_int) -> *c_char;
    fn OGR_F_GetFieldAsDouble(hFeat: *(), iField: c_int) -> c_double;
    fn OGR_F_GetGeometryRef(hFeat: *()) -> *();
    fn OGR_F_Destroy(hFeat: *());
    fn OGR_G_ExportToWkt(hGeom: *(), ppszSrcText: **c_char) -> c_int;
    fn OGR_Fld_GetType(hDefn: *()) -> c_int;
    fn OGRFree(ptr: *());
}

static OFTInteger:        c_int = 0;
static OFTIntegerList:    c_int = 1;
static OFTReal:           c_int = 2;
static OFTRealList:       c_int = 3;
static OFTString:         c_int = 4;
static OFTStringList:     c_int = 5;
static OFTWideString:     c_int = 6;
static OFTWideStringList: c_int = 7;
static OFTBinary:         c_int = 8;
static OFTDate:           c_int = 9;
static OFTTime:           c_int = 10;
static OFTDateTime:       c_int = 11;

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
    pub fn layer_count(&self) -> int {
        return unsafe { OGR_DS_GetLayerCount(self.c_dataset) } as int;
    }

    pub fn layer<'a>(&'a self, idx: int) -> Option<Layer<'a>> {
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
    pub fn field(&self, name: String) -> Option<FieldValue> {
        return name.with_c_str(|c_name| unsafe {
            let field_id = OGR_F_GetFieldIndex(self.c_feature, c_name);
            if field_id == -1 {
                return None;
            }
            let field_defn = OGR_F_GetFieldDefnRef(self.c_feature, field_id);
            let field_type = OGR_Fld_GetType(field_defn);
            return match field_type {
                OFTString => {
                    let rv = OGR_F_GetFieldAsString(self.c_feature, field_id);
                    return Some(StringValue(raw::from_c_str(rv)));
                },
                OFTReal => {
                    let rv = OGR_F_GetFieldAsDouble(self.c_feature, field_id);
                    return Some(F64Value(rv as f64));
                },
                _ => fail!("Unknown field type {}", field_type)
            }
        });
    }

    pub fn wkt(&self) -> String {
        unsafe {
            let c_geom = OGR_F_GetGeometryRef(self.c_feature);
            let c_wkt: *c_char = null();
            OGR_G_ExportToWkt(c_geom, &c_wkt);
            let wkt = raw::from_c_str(c_wkt);
            OGRFree(c_wkt as *());
            return wkt;
        }
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


pub enum FieldValue {
    StringValue(String),
    F64Value(f64),
}


impl FieldValue {
    pub fn as_string(self) -> String {
        match self { StringValue(rv) => rv, _ => fail!("not a string") }
    }

    pub fn as_f64(self) -> f64 {
        match self { F64Value(rv) => rv, _ => fail!("not an f64") }
    }
}


#[cfg(test)]
mod test {
    use std::os::getenv;
    use std::path::Path;
    use super::{Feature, FeatureIterator, open};


    fn fixture_path(name: &str) -> Path {
        let envvar = "RUST_GDAL_TEST_FIXTURES";
        let fixtures = match getenv(envvar) {
            Some(p) => Path::new(p),
            None => fail!("Environment variable {} not set", envvar)
        };
        let rv = fixtures.join(name);
        return rv;
    }


    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.00001);
        assert!(f > 0.99999);
    }


    #[test]
    fn test_layer_count() {
        let ds = open(&fixture_path("roads.geojson")).unwrap();
        assert_eq!(ds.layer_count(), 1);
    }


    fn with_features(fixture: &str, f: |FeatureIterator|) {
        let ds = open(&fixture_path(fixture)).unwrap();
        let layer = ds.layer(0).unwrap();
        f(layer.features());
    }


    fn with_first_feature(fixture: &str, f: |Feature|) {
        with_features(fixture, |mut features| f(features.next().unwrap()));
    }


    #[test]
    fn test_iterate_features() {
        with_features("roads.geojson", |mut features| {
            let feature_vec: Vec<Feature> = features.collect();
            assert_eq!(feature_vec.len(), 21);
        });
    }


    #[test]
    fn test_string_field() {
        with_features("roads.geojson", |mut features| {
            let feature = features.next().unwrap();
            assert_eq!(feature.field("highway".to_string())
                              .unwrap()
                              .as_string(),
                       "footway".to_string());
            assert_eq!(
                features.count(|field| {
                    let highway = field.field("highway".to_string())
                                       .unwrap()
                                       .as_string();
                    highway == "residential".to_string() }),
                2);
        });
    }


    #[test]
    fn test_float_field() {
        with_first_feature("roads.geojson", |feature| {
            assert_almost_eq(
                feature.field("sort_key".to_string())
                       .unwrap()
                       .as_f64(),
                -9.0
            );
        });
    }


    #[test]
    fn test_missing_field() {
        with_first_feature("roads.geojson", |feature| {
            assert!(feature.field("no such field".to_string()).is_none());
        });
    }


    #[test]
    fn test_wkt() {
        with_first_feature("roads.geojson", |feature| {
            let wkt = feature.wkt();
            let wkt_ok = format!("{}{}",
                "LINESTRING (26.1019276 44.4302748,",
                "26.1019382 44.4303191,26.1020002 44.4304202)"
                ).to_string();
            assert_eq!(wkt, wkt_ok);
        });
    }
}
