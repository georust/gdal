#[cfg(feature = "curl-sys")]
extern crate curl_sys;
#[cfg(feature = "geos")]
extern crate geos_sys;
#[cfg(feature = "DRIVER_HDF5")]
extern crate hdf5_src;
#[cfg(feature = "DRIVER_SQLITE")]
extern crate libsqlite3_sys;
#[cfg(feature = "DRIVER_NETCDF")]
extern crate netcdf_src;
#[cfg(feature = "DRIVER_PG")]
extern crate pq_src;

extern crate proj_sys;
extern crate link_cplusplus;
