#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]
#![allow(rustdoc::bare_urls)]

#[cfg(feature = "bundled")]
extern crate gdal_src;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
