#[cfg(feature = "curl-sys")]
extern crate curl_sys;
#[cfg(feature = "geos")]
extern crate geos_sys;
#[cfg(feature = "driver_hdf5")]
extern crate hdf5_src;
#[cfg(feature = "driver_sqlite")]
extern crate libsqlite3_sys;
#[cfg(feature = "driver_netcdf")]
extern crate netcdf_src;
#[cfg(feature = "driver_pg")]
extern crate pq_src;

extern crate link_cplusplus;
extern crate proj_sys;
