extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=gdal");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .blacklist_type("FP_NAN")
        .blacklist_type("FP_INFINITE")
        .blacklist_type("FP_ZERO")
        .blacklist_type("FP_SUBNORMAL")
        .blacklist_type("FP_NORMAL")
        .prepend_enum_name(false)
        .constified_enum_module("CPLErr")
        .constified_enum_module("OGRwkbGeometryType")
        .constified_enum_module("OGRErr")
        .constified_enum_module("OGRFieldType")
        .ctypes_prefix("libc")
        .whitelist_function("CPL.*")
        .whitelist_function("GDAL.*")
        .whitelist_function("OGR.*")
        .whitelist_function("OSR.*")
        .whitelist_function("OCT.*")
        .whitelist_function("VSI.*")
        .whitelist_var("OGR.*")
        .whitelist_var("FALSE")
        .whitelist_var("TRUE")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
