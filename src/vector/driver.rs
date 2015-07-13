use std::sync::{Once, ONCE_INIT};
use vector::ogr;

static START: Once = ONCE_INIT;
static mut registered_drivers: bool = false;


pub fn _register_drivers() {
    unsafe {
        START.call_once(|| {
            ogr::OGRRegisterAll();
            registered_drivers = true;
        });
    }
}
