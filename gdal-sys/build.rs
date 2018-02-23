#[cfg(feature = "bindgen")]
extern crate bindgen;
extern crate pkg_config;

use pkg_config::{Config, Library};
use std::env;
use std::path::PathBuf;

fn find_gdal() -> Option<Library> {
    Config::new().probe("gdal").ok()
}

#[cfg(feature = "bindgen")]
pub fn build(gdal: Option<Library>, out_path: &std::path::Path) {
    #[cfg(windows)]
    {
        // get the path to GDAL_HOME
        let home_path = env::var("GDAL_HOME").expect("Environment variable $GDAL_HOME not found!");

        // detect the path to gdal_i.lib (works for MSVC and GNU)
        let lib_suffix = "_i";
        let lib_search_path = Path::new(&home_path).join("lib");
        let lib_path = lib_search_path.join(&format!("{}{}.lib", lib_name, lib_suffix));

        if lib_search_path.exists() && lib_path.exists() {
            println!(
                "cargo:rustc-link-search={}",
                lib_search_path.to_string_lossy()
            );
            println!(
                "cargo:rustc-link-lib={}={}",
                link_type,
                format!("{}{}", lib_name, lib_suffix)
            );
        } else {
            #[cfg(target_env = "msvc")]
            {
                panic!("windows-msvc requires gdal_i.lib to be found in $GDAL_HOME\\lib.");
            }

            #[cfg(target_env = "gnu")]
            {
                // detect if a gdal{version}.dll is available
                let versions = [201, 200, 111, 110];
                let bin_path = Path::new(&home_path).join("bin");
                if let Some(version) = versions
                    .iter()
                    .find(|v| bin_path.join(&format!("{}{}.dll", lib_name, v)).exists())
                {
                    println!("cargo:rustc-link-search={}", bin_path.to_string_lossy());
                    println!(
                        "cargo:rustc-link-lib={}={}",
                        link_type,
                        format!("{}{}", lib_name, version)
                    );
                } else {
                    panic!("windows-gnu requires either gdal_i.lib in $GDAL_HOME\\lib OR gdal{version}.dll in $GDAL_HOME\\bin.");
                }
            }
        }
    }

    let mut builder = bindgen::Builder::default();
    if let Some(gdal) = gdal {
        for path in &gdal.include_paths {
            builder = builder.clang_arg("-I");
            builder = builder.clang_arg(path.to_str().unwrap());
        }
    }

    builder
        .header("wrapper.h")
        .constified_enum_module(".*")
        .ctypes_prefix("libc")
        .whitelist_function("CPL.*")
        .whitelist_function("GDAL.*")
        .whitelist_function("OGR.*")
        .whitelist_function("OSR.*")
        .whitelist_function("OCT.*")
        .whitelist_function("VSI.*")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path)
        .expect("Unable to write bindings to file");
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    #[cfg(feature = "bindgen")]
    build(find_gdal(), &out_path);

    #[cfg(not(feature = "bindgen"))]
    {
        find_gdal();

        let prebuilt_paths = &["prebuilt-bindings/gdal_2.2.rs"];
        std::fs::copy(&prebuilt_paths[prebuilt_paths.len() - 1], &out_path)
            .expect("Can't copy bindings to output directory");
    }
}
