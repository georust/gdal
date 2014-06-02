use std::ptr::null;
use libc::{c_int, c_char};
use sync::mutex::{StaticMutex, MUTEX_INIT};


#[link(name="gdal")]
extern {
    fn OGRRegisterAll();
    fn OGROpen(pszName: *c_char, bUpdate: c_int, pahDriverList: *()) -> *();
    fn OGR_DS_GetLayerCount(OGRDataSourceH: *()) -> c_int;
    fn OGR_DS_Destroy(OGRDataSourceH: *());
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
}


impl Drop for VectorDataset {
    fn drop(&mut self) {
        unsafe { OGR_DS_Destroy(self.c_dataset); }
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
    use super::open;


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
}
