use semver::Version;

use pkg_config::Config;
use std::env;
use std::path::PathBuf;

#[cfg(feature = "bindgen")]
pub fn write_bindings(include_paths: Vec<String>, out_path: &Path) {
    let mut builder = bindgen::Builder::default()
        .size_t_is_usize(true)
        .header("wrapper.h")
        .constified_enum_module(".*")
        .ctypes_prefix("libc")
        .allowlist_function("CPL.*")
        .allowlist_function("CSL.*")
        .allowlist_function("GDAL.*")
        .allowlist_function("OGR.*")
        .allowlist_function("OSR.*")
        .allowlist_function("OCT.*")
        .allowlist_function("VSI.*");

    for path in include_paths {
        builder = builder
            .clang_arg("-I")
            .clang_arg(path)
            .clang_arg("-fretain-comments-from-system-headers");
    }

    builder
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path)
        .expect("Unable to write bindings to file");
}

fn env_dir(var: &str) -> Option<PathBuf> {
    let dir = env::var_os(var).map(PathBuf::from);

    if let Some(ref dir) = dir {
        if !dir.exists() {
            panic!("{} was set to {}, which doesn't exist.", var, dir.display());
        }
    }

    dir
}

fn add_docs_rs_helper(version: Option<&str>) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("docs_rs_helper.rs");
    let docs_helper_code = if let Some(version) = version {
        format!(
            r#"
        pub fn gdal_version_docs_rs_wrapper() -> String {{
            "{version}".to_string()
        }}"#
        )
    } else {
        r#"
        pub fn gdal_version_docs_rs_wrapper() -> String {
            let key = "VERSION_NUM";
            let c_key = std::ffi::CString::new(key.as_bytes()).unwrap();

            unsafe {
                let res_ptr = crate::GDALVersionInfo(c_key.as_ptr());
                let c_res = std::ffi::CStr::from_ptr(res_ptr);
                c_res.to_string_lossy().into_owned()
            }
        }"#
        .to_string()
    };
    std::fs::write(out_path, docs_helper_code.as_bytes()).unwrap();
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    // Hardcode a prebuilt binding version while generating docs.
    // Otherwise docs.rs will explode due to not actually having libgdal installed.
    if std::env::var("DOCS_RS").is_ok() {
        let version = Version::parse("3.5.0").expect("invalid version for docs.rs");
        add_docs_rs_helper(Some("3050000"));
        println!(
            "cargo:rustc-cfg=gdal_sys_{}_{}_{}",
            version.major, version.minor, version.patch
        );

        let binding_path = PathBuf::from(format!(
            "prebuilt-bindings/gdal_{}.{}.rs",
            version.major, version.minor
        ));

        if !binding_path.exists() {
            panic!("Missing bindings for docs.rs (version {})", version);
        }

        std::fs::copy(&binding_path, &out_path).expect("Can't copy bindings to output directory");

        return;
    }

    add_docs_rs_helper(None);
    println!("cargo:rerun-if-env-changed=GDAL_STATIC");
    println!("cargo:rerun-if-env-changed=GDAL_DYNAMIC");
    println!("cargo:rerun-if-env-changed=GDAL_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=GDAL_LIB_DIR");
    println!("cargo:rerun-if-env-changed=GDAL_HOME");
    println!("cargo:rerun-if-env-changed=GDAL_VERSION");

    let mut need_metadata = true;
    let lib_name = String::from("gdal");

    let prefer_static =
        env::var_os("GDAL_STATIC").is_some() && env::var_os("GDAL_DYNAMIC").is_none();

    let mut include_dir = env_dir("GDAL_INCLUDE_DIR");
    let mut lib_dir = env_dir("GDAL_LIB_DIR");
    let home_dir = env_dir("GDAL_HOME");
    let mut version = env::var_os("GDAL_VERSION")
        .map(|vs| vs.to_string_lossy().to_string())
        .and_then(|vs| Version::parse(vs.trim()).ok());

    if let Some(ref home_dir) = home_dir {
        if include_dir.is_none() {
            let dir = home_dir.join("include");
            if cfg!(feature = "bindgen") && !dir.exists() {
                panic!(
                    "bindgen was enabled, but GDAL_INCLUDE_DIR was not set and {} doesn't exist.",
                    dir.display()
                );
            }
            include_dir = Some(dir);
        }

        if lib_dir.is_none() {
            let dir = home_dir.join("lib");
            if !dir.exists() {
                panic!(
                    "GDAL_LIB_DIR was not set and {} doesn't exist.",
                    dir.display()
                );
            }
            lib_dir = Some(dir);
        }
    }

    if let Some(lib_dir) = lib_dir {
        let link_type = if prefer_static { "static" } else { "dylib" };

        println!("cargo:rustc-link-lib={link_type}={lib_name}");
        println!("cargo:rustc-link-search={}", lib_dir.to_str().unwrap());

        need_metadata = false;
    }

    let mut include_paths = Vec::new();
    if let Some(ref dir) = include_dir {
        include_paths.push(dir.as_path().to_str().unwrap().to_string());
    }

    let gdal_pkg_config = Config::new()
        .statik(prefer_static)
        .cargo_metadata(need_metadata)
        .probe("gdal");

    if let Ok(gdal) = &gdal_pkg_config {
        for dir in &gdal.include_paths {
            include_paths.push(dir.to_str().unwrap().to_string());
        }
        if version.is_none() {
            if let Ok(pkg_version) = Version::parse(gdal.version.trim()) {
                version.replace(pkg_version);
            }
        }
    }

    #[cfg(feature = "bindgen")]
    write_bindings(include_paths, &out_path);

    #[cfg(not(feature = "bindgen"))]
    {
        if let Some(version) = version {
            println!(
                "cargo:rustc-cfg=gdal_sys_{}_{}_{}",
                version.major, version.minor, version.patch
            );

            let binding_path = PathBuf::from(format!(
                "prebuilt-bindings/gdal_{}.{}.rs",
                version.major, version.minor
            ));
            if !binding_path.exists() {
                panic!("No pre-built bindings available for GDAL version {}.{}. Use `--features bindgen` to generate your own bindings.", version.major, version.minor);
            }

            std::fs::copy(&binding_path, &out_path)
                .expect("Can't copy bindings to output directory");
        } else if let Err(pkg_config_err) = &gdal_pkg_config {
            // Special case output for this common error
            if matches!(pkg_config_err, pkg_config::Error::Command { cause, .. } if cause.kind() == std::io::ErrorKind::NotFound)
            {
                panic!("Could not find `pkg-config` in your path. Please install it before building gdal-sys.");
            } else {
                panic!("Error while running `pkg-config`: {}", pkg_config_err);
            }
        } else {
            panic!("No GDAL version detected");
        }
    }
}
