#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]
#![allow(rustdoc::bare_urls)]
// bindgen test code generates lots of warnings when testing
// sizes of types due to using constructs such as:
// `unsafe { &(*(::std::ptr::null::<__sbuf>()))._base as *const _ as usize }`
// This disables those warnings.
#![cfg_attr(test, allow(deref_nullptr))]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
include!(concat!(env!("OUT_DIR"), "/docs_rs_helper.rs"));
