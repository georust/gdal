

fn main() {

    let lib_name = "gdal";
    let link_type = if env::var_os("GDAL_LIB_STATIC").is_some() {
        "static"
    } else {
        "dylib"
    };
    println!("cargo:rerun-if-env-changed=GDAL_LIB_STATIC");
    println!("cargo:rerun-if-env-changed=GDAL_HOME");

    #[cfg(windows)]
    {
        use std::path::Path;

        // get the path to GDAL_HOME
        let home_path = env::var("GDAL_HOME").expect("Environment variable $GDAL_HOME not found!");

        // detect the path to gdal_i.lib (works for MSVC and GNU)
        let lib_suffix = "_i";
        let lib_search_path = Path::new(&home_path).join("lib");
        let lib_path = lib_search_path.join(&format!("{}{}.lib", lib_name, lib_suffix));

        if lib_search_path.exists() && lib_path.exists() {
            println!("cargo:rustc-link-search={}", lib_search_path.to_string_lossy());
            println!("cargo:rustc-link-lib={}={}", link_type, format!("{}{}",lib_name, lib_suffix));
        } else {

            #[cfg(target_env="msvc")]
            {
                panic!("windows-msvc requires gdal_i.lib to be found in $GDAL_HOME\\lib.");
            }

            #[cfg(target_env="gnu")]
            {
                // detect if a gdal{version}.dll is available
                let versions = [201, 200, 111, 110];
                let bin_path = Path::new(&home_path).join("bin");
                if let Some(version) = versions.iter().find(|v| bin_path.join(&format!("{}{}.dll", lib_name, v)).exists()){
                    println!("cargo:rustc-link-search={}", bin_path.to_string_lossy());
                    println!("cargo:rustc-link-lib={}={}", link_type, format!("{}{}",lib_name, version));
                }
                else {
                    panic!("windows-gnu requires either gdal_i.lib in $GDAL_HOME\\lib OR gdal{version}.dll in $GDAL_HOME\\bin.");
                }
            }
        }
    }

    #[cfg(unix)]
    {
        if let Ok(home) = env::var("GDAL_HOME") {
            println!("cargo:rustc-link-search={}", home);
        }
        println!("cargo:rustc-link-lib={}={}", link_type, lib_name);
    }
}
