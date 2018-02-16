extern crate bindgen;
extern crate pkg_config;

use bindgen::Builder;
use pkg_config::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    let mut builder = Builder::default();

    let gdal = Config::new().probe("gdal").unwrap();
    for path in &gdal.libs {
        println!("cargo:rustc-link-lib={}", path);
    }
    for path in &gdal.include_paths {
        builder = builder.clang_arg("-I");
        builder = builder.clang_arg(path.to_str().unwrap());
    }

    let bindings = builder
        .header("wrapper.h")
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
