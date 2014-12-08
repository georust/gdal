use std::ptr::null;
use libc::{c_int, c_char, c_double};
use rustrt::mutex::{StaticNativeMutex, NATIVE_MUTEX_INIT};
use utils::_string;


#[link(name="gdal")]
extern {
    fn OGRRegisterAll();
    fn OGROpen(pszName: *const c_char, bUpdate: c_int, pahDriverList: *const ()) -> *const ();
    fn OGR_DS_GetLayerCount(hDS: *const ()) -> c_int;
    fn OGR_DS_Destroy(hDataSource: *const ());
    fn OGR_DS_GetLayer(hDS: *const (), iLayer: c_int) -> *const ();
    fn OGR_L_GetLayerDefn(hLayer: *const ()) -> *const ();
    fn OGR_L_GetNextFeature(hLayer: *const ()) -> *const ();
    fn OGR_FD_GetFieldCount(hDefn: *const ()) -> c_int;
    fn OGR_FD_GetFieldDefn(hDefn: *const (), iField: c_int) -> *const ();
    fn OGR_F_GetFieldIndex(hFeat: *const (), pszName: *const c_char) -> c_int;
    fn OGR_F_GetFieldDefnRef(hFeat: *const (), i: c_int) -> *const ();
    fn OGR_F_GetFieldAsString(hFeat: *const (), iField: c_int) -> *const c_char;
    fn OGR_F_GetFieldAsDouble(hFeat: *const (), iField: c_int) -> c_double;
    fn OGR_F_GetGeometryRef(hFeat: *const ()) -> *const ();
    fn OGR_F_Destroy(hFeat: *const ());
    fn OGR_G_ExportToWkt(hGeom: *const (), ppszSrcText: *const *const c_char) -> c_int;
    fn OGR_G_ExportToJson(hGeometry: *const ()) -> *const c_char;
    fn OGR_Fld_GetNameRef(hDefn: *const ()) -> *const c_char;
    fn OGR_Fld_GetType(hDefn: *const ()) -> c_int;
    fn OGRFree(ptr: *const ());
    fn VSIFree(ptr: *const ());
}

const OFT_REAL:             c_int = 2;
const OFT_STRING:           c_int = 4;

static mut LOCK: StaticNativeMutex = NATIVE_MUTEX_INIT;
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
    c_dataset: *const (),
}


impl VectorDataset {
    pub fn count(&self) -> int {
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
    c_layer: *const (),
}


impl<'a> Layer<'a> {
    pub fn fields<'a>(&'a self) -> FieldIterator<'a> {
        let c_feature_defn = unsafe { OGR_L_GetLayerDefn(self.c_layer) };
        let total = unsafe { OGR_FD_GetFieldCount(c_feature_defn) } as int;
        return FieldIterator{
            layer: self,
            c_feature_defn: c_feature_defn,
            next_id: 0,
            total: total
        };
    }

    pub fn features<'a>(&'a self) -> FeatureIterator<'a> {
        return FeatureIterator{layer: self};
    }
}


pub struct FieldIterator<'a> {
    layer: &'a Layer<'a>,
    c_feature_defn: *const (),
    next_id: int,
    total: int,
}


impl<'a> Iterator<Field<'a>> for FieldIterator<'a> {
    #[inline]
    fn next(&mut self) -> Option<Field<'a>> {
        if self.next_id == self.total {
            return None;
        }
        let field = Field{
            layer: self.layer,
            c_field_defn: unsafe { OGR_FD_GetFieldDefn(
                self.c_feature_defn,
                self.next_id as c_int
            ) }
        };
        self.next_id += 1;
        return Some(field);
    }
}


pub struct Field<'a> {
    layer: &'a Layer<'a>,
    c_field_defn: *const (),
}


impl<'a> Field<'a> {
    pub fn name(&'a self) -> String {
        let rv = unsafe { OGR_Fld_GetNameRef(self.c_field_defn) };
        return _string(rv);
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
    c_feature: *const (),
}


impl<'a> Feature<'a> {
    pub fn field(&self, name: &str) -> Option<FieldValue> {
        return name.with_c_str(|c_name| unsafe {
            let field_id = OGR_F_GetFieldIndex(self.c_feature, c_name);
            if field_id == -1 {
                return None;
            }
            let field_defn = OGR_F_GetFieldDefnRef(self.c_feature, field_id);
            let field_type = OGR_Fld_GetType(field_defn);
            return match field_type {
                OFT_STRING => {
                    let rv = OGR_F_GetFieldAsString(self.c_feature, field_id);
                    return Some(FieldValue::StringValue(_string(rv)));
                },
                OFT_REAL => {
                    let rv = OGR_F_GetFieldAsDouble(self.c_feature, field_id);
                    return Some(FieldValue::RealValue(rv as f64));
                },
                _ => panic!("Unknown field type {}", field_type)
            }
        });
    }

    pub fn wkt(&self) -> String {
        unsafe {
            let c_geom = OGR_F_GetGeometryRef(self.c_feature);
            let c_wkt: *const c_char = null();
            OGR_G_ExportToWkt(c_geom, &c_wkt);
            let wkt = _string(c_wkt);
            OGRFree(c_wkt as *const ());
            return wkt;
        }
    }


    pub fn json(&self) -> String {
        unsafe {
            let c_geom = OGR_F_GetGeometryRef(self.c_feature);
            let c_json = OGR_G_ExportToJson(c_geom);
            let json = _string(c_json);
            VSIFree(c_json as *const ());
            return json;
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
    RealValue(f64),
}


impl FieldValue {
    pub fn as_string(self) -> String {
        match self {
            FieldValue::StringValue(rv) => rv,
            _ => panic!("not a StringValue")
        }
    }

    pub fn as_real(self) -> f64 {
        match self {
            FieldValue::RealValue(rv) => rv,
            _ => panic!("not a RealValue")
        }
    }
}


#[cfg(test)]
mod test {
    use std::path::Path;
    use super::{Feature, FeatureIterator, open};


    fn fixtures() -> Path {
        return Path::new(file!()).dir_path().dir_path().join("fixtures");
    }


    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.00001);
        assert!(f > 0.99999);
    }


    #[test]
    fn test_layer_count() {
        let ds = open(&fixtures().join("roads.geojson")).unwrap();
        assert_eq!(ds.count(), 1);
    }


    fn with_features(fixture: &str, f: |FeatureIterator|) {
        let ds = open(&fixtures().join(fixture)).unwrap();
        let layer = ds.layer(0).unwrap();
        f(layer.features());
    }


    fn with_first_feature(fixture: &str, f: |Feature|) {
        with_features(fixture, |mut features| f(features.next().unwrap()));
    }


    #[test]
    fn test_iterate_features() {
        with_features("roads.geojson", |features| {
            let feature_vec: Vec<Feature> = features.collect();
            assert_eq!(feature_vec.len(), 21);
        });
    }


    #[test]
    fn test_string_field() {
        with_features("roads.geojson", |mut features| {
            let feature = features.next().unwrap();
            assert_eq!(feature.field("highway")
                              .unwrap()
                              .as_string(),
                       "footway".to_string());
            assert_eq!(
                features.filter(|field| {
                    let highway = field.field("highway")
                                       .unwrap()
                                       .as_string();
                    highway == "residential".to_string() })
                    .count(),
                2);
        });
    }


    #[test]
    fn test_float_field() {
        with_first_feature("roads.geojson", |feature| {
            assert_almost_eq(
                feature.field("sort_key")
                       .unwrap()
                       .as_real(),
                -9.0
            );
        });
    }


    #[test]
    fn test_missing_field() {
        with_first_feature("roads.geojson", |feature| {
            assert!(feature.field("no such field").is_none());
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


    #[test]
    fn test_json() {
        with_first_feature("roads.geojson", |feature| {
            let json = feature.json();
            let json_ok = format!("{}{}{}{}",
                "{ \"type\": \"LineString\", \"coordinates\": [ ",
                "[ 26.1019276, 44.4302748 ], ",
                "[ 26.1019382, 44.4303191 ], ",
                "[ 26.1020002, 44.4304202 ] ] }"
                ).to_string();
            assert_eq!(json, json_ok);
        });
    }


    #[test]
    fn test_schema() {
        let ds = open(&fixtures().join("roads.geojson")).unwrap();
        let layer = ds.layer(0).unwrap();
        let name_list: Vec<String> = layer
            .fields()
            .map(|f| f.name())
            .collect();
        let ok_names: Vec<String> = vec!(
            "kind", "sort_key", "is_link", "is_tunnel",
            "is_bridge", "railway", "highway")
            .iter().map(|s| s.to_string()).collect();
        assert_eq!(name_list, ok_names);
    }
}
