use std::sync::{StaticMutex, MUTEX_INIT};
use std::ffi::CString;
use std::path::Path;
use std::ptr::null;
use libc::c_int;
use vector::{ogr, Layer};


static mut LOCK: StaticMutex = MUTEX_INIT;
static mut registered_drivers: bool = false;


fn register_drivers() {
    unsafe {
        let _g = LOCK.lock();
        if ! registered_drivers {
            ogr::OGRRegisterAll();
            registered_drivers = true;
        }
    }
}


pub struct Dataset {
    c_dataset: *const (),
}


impl Dataset {
    pub fn open(path: &Path) -> Option<Dataset> {
        register_drivers();
        let filename = path.to_str().unwrap();
        let c_filename = CString::new(filename.as_bytes()).unwrap();
        let c_dataset = unsafe { ogr::OGROpen(c_filename.as_ptr(), 0, null()) };
        return match c_dataset.is_null() {
            true  => None,
            false => Some(Dataset{c_dataset: c_dataset}),
        };
    }

    pub fn count(&self) -> isize {
        return unsafe { ogr::OGR_DS_GetLayerCount(self.c_dataset) } as isize;
    }

    pub fn layer<'a>(&'a self, idx: isize) -> Option<Layer<'a>> {
        let c_layer = unsafe { ogr::OGR_DS_GetLayer(self.c_dataset, idx as c_int) };
        return match c_layer.is_null() {
            true  => None,
            false => Some(unsafe { Layer::_with_dataset(self, c_layer) }),
        };
    }
}


impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe { ogr::OGR_DS_Destroy(self.c_dataset); }
    }
}
