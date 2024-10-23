use std::path::PathBuf;

macro_rules! handle_ogr_driver {
    ($config: ident, $driver: literal) => {
        if cfg!(feature = $driver) {
            $config.define(format!("OGR_ENABLE_{}", $driver.to_ascii_uppercase()), "ON");
        } else {
            $config.define(
                format!("OGR_ENABLE_{}", $driver.to_ascii_uppercase()),
                "OFF",
            );
        }
    };
}

macro_rules! handle_gdal_driver {
    ($config: ident, $driver: literal) => {
        if cfg!(feature = $driver) {
            $config.define(
                format!("GDAL_ENABLE_{}", $driver.to_ascii_uppercase()),
                "ON",
            );
        } else {
            $config.define(
                format!("GDAL_ENABLE_{}", $driver.to_ascii_uppercase()),
                "OFF",
            );
        }
    };
}

fn find_library(lib_name: &str, path: impl Into<std::path::PathBuf>) -> PathBuf {
    let path = path.into();
    if path.join("lib64").join(format!("lib{lib_name}.a")).exists() {
        path.join("lib64").join(format!("lib{lib_name}.a"))
    } else if path.join("lib").join(format!("lib{lib_name}.a")).exists() {
        path.join("lib").join(format!("lib{lib_name}.a"))
    } else if path.join("lib").join(format!("{lib_name}.lib")).exists() {
        path.join("lib").join(format!("{lib_name}.lib"))
    } else {
        panic!("{lib_name} not found in {}", path.display());
    }
}

fn main() {
    if cfg!(feature = "nobuild") {
        return;
    }
    // gdal doesn't like non clean builds so we remove any artifact from an older build
    // https://github.com/OSGeo/gdal/issues/10125
    // This hopefully does not break all the caching as we don't rerun the build script
    // every time, just if something changed. In that case we will do always a clean build
    // and not an incremental rebuild because of the gdal build system seems not to be able
    // to handle that well
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("Set by cargo"));
    if out_dir.exists() {
        let _ = std::fs::remove_dir_all(&out_dir);
        let _ = std::fs::create_dir_all(&out_dir);
    }

    let proj_root =
        std::path::PathBuf::from(std::env::var("DEP_PROJ_ROOT").expect("set by proj-sys"));
    let proj_library = if std::env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("windows") {
        if proj_root.join("lib").join("proj_d.lib").exists() {
            proj_root.join("lib").join("proj_d.lib")
        } else {
            proj_root.join("lib").join("proj.lib")
        }
    } else {
        find_library("proj", &proj_root)
    };
    let proj_include = proj_root.join("include");

    let mut config = cmake::Config::new("source");

    config
        .define("GDAL_BUILD_OPTIONAL_DRIVERS", "OFF")
        .define("OGR_BUILD_OPTIONAL_DRIVERS", "OFF")
        .define("GDAL_USE_INTERNAL_LIBS", "ON")
        .define("GDAL_USE_EXTERNAL_LIBS", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_STATIC_LIBS", "ON")
        .define("BUILD_APPS", "OFF")
        .define("BUILD_DOCS", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_GMOCK", "OFF")
        .define("GDAL_FIND_PACKAGE_PROJ_MODE", "MODULE")
        .define("PROJ_INCLUDE_DIR", print_path(&proj_include))
        .define("PROJ_LIBRARY", print_path(&proj_library))
        .define("ACCEPT_MISSING_LINUX_FS_HEADER", "ON");
    // enable the gpkg driver

    // simple drivers without external dependencies
    handle_ogr_driver!(config, "driver_avc");
    handle_ogr_driver!(config, "driver_cad");
    handle_ogr_driver!(config, "driver_csv");
    handle_ogr_driver!(config, "driver_dgn");
    handle_ogr_driver!(config, "driver_dxf");
    handle_ogr_driver!(config, "driver_edigeo");
    handle_ogr_driver!(config, "driver_flatgeobuf");
    handle_ogr_driver!(config, "driver_geoconcept");
    handle_ogr_driver!(config, "driver_geojson");
    handle_ogr_driver!(config, "driver_gmt");
    handle_ogr_driver!(config, "driver_gtfs");
    handle_ogr_driver!(config, "driver_jsonfg");
    handle_ogr_driver!(config, "driver_mapml");
    handle_ogr_driver!(config, "driver_openfilegdb");
    handle_ogr_driver!(config, "driver_pgdump");
    handle_ogr_driver!(config, "driver_ntf");
    handle_ogr_driver!(config, "driver_s57");
    handle_ogr_driver!(config, "driver_selafin");
    handle_ogr_driver!(config, "driver_shape");
    handle_ogr_driver!(config, "driver_sxf");
    handle_ogr_driver!(config, "driver_tab");
    handle_ogr_driver!(config, "driver_tiger");
    handle_ogr_driver!(config, "driver_vdv");
    handle_ogr_driver!(config, "driver_wasp");
    handle_ogr_driver!(config, "driver_idrisi");
    handle_ogr_driver!(config, "driver_sdts");
    handle_ogr_driver!(config, "driver_vrt");
    handle_ogr_driver!(config, "driver_mem");

    handle_gdal_driver!(config, "driver_aaigrid");
    handle_gdal_driver!(config, "driver_adrg");
    handle_gdal_driver!(config, "driver_aigrid");
    handle_gdal_driver!(config, "driver_airsar");
    handle_gdal_driver!(config, "driver_blx");
    handle_gdal_driver!(config, "driver_bmp");
    handle_gdal_driver!(config, "driver_bsb");
    handle_gdal_driver!(config, "driver_cals");
    handle_gdal_driver!(config, "driver_ceos");
    handle_gdal_driver!(config, "driver_coasp");
    handle_gdal_driver!(config, "driver_cosar");
    handle_gdal_driver!(config, "driver_ctg");
    handle_gdal_driver!(config, "driver_derived");
    handle_gdal_driver!(config, "driver_dimap");
    handle_gdal_driver!(config, "driver_dted");
    handle_gdal_driver!(config, "driver_elas");
    handle_gdal_driver!(config, "driver_envisat");
    handle_gdal_driver!(config, "driver_ers");
    handle_gdal_driver!(config, "driver_fit");
    handle_gdal_driver!(config, "driver_gff");
    handle_gdal_driver!(config, "driver_gif");
    handle_gdal_driver!(config, "driver_grib");
    handle_gdal_driver!(config, "driver_gsg");
    handle_gdal_driver!(config, "driver_gtiff");
    handle_gdal_driver!(config, "driver_gxf");
    handle_gdal_driver!(config, "driver_hf2");
    handle_gdal_driver!(config, "driver_hfa");
    handle_gdal_driver!(config, "driver_ilwis");
    handle_gdal_driver!(config, "driver_iris");
    handle_gdal_driver!(config, "driver_jaxapalsar");
    handle_gdal_driver!(config, "driver_jdem");
    handle_gdal_driver!(config, "driver_jpeg");
    handle_gdal_driver!(config, "driver_kmlsuperoverlay");
    handle_gdal_driver!(config, "driver_l1b");
    handle_gdal_driver!(config, "driver_leveller");
    handle_gdal_driver!(config, "driver_map");
    handle_gdal_driver!(config, "driver_mrf");
    handle_gdal_driver!(config, "driver_msgn");
    handle_gdal_driver!(config, "driver_ngsgeoid");
    handle_gdal_driver!(config, "driver_nift");
    handle_gdal_driver!(config, "driver_northwood");
    handle_gdal_driver!(config, "driver_ozi");
    handle_gdal_driver!(config, "driver_pcidsk");
    handle_gdal_driver!(config, "driver_pcraster");
    handle_gdal_driver!(config, "driver_png");
    handle_gdal_driver!(config, "driver_prf");
    handle_gdal_driver!(config, "driver_r");
    handle_gdal_driver!(config, "driver_raw");
    handle_gdal_driver!(config, "driver_rik");
    handle_gdal_driver!(config, "driver_rmf");
    handle_gdal_driver!(config, "driver_rs2");
    handle_gdal_driver!(config, "driver_safe");
    handle_gdal_driver!(config, "driver_saga");
    handle_gdal_driver!(config, "driver_sar_ceos");
    handle_gdal_driver!(config, "driver_sentinel2");
    handle_gdal_driver!(config, "driver_sgi");
    handle_gdal_driver!(config, "driver_sigdem");
    handle_gdal_driver!(config, "driver_srtmhgt");
    handle_gdal_driver!(config, "driver_stacit");
    handle_gdal_driver!(config, "driver_stacta");
    handle_gdal_driver!(config, "driver_terragen");
    handle_gdal_driver!(config, "driver_tga");
    handle_gdal_driver!(config, "driver_til");
    handle_gdal_driver!(config, "driver_tsx");
    handle_gdal_driver!(config, "driver_usgsdem");
    handle_gdal_driver!(config, "driver_xpm");
    handle_gdal_driver!(config, "driver_xyz");
    handle_gdal_driver!(config, "driver_zmap");
    handle_gdal_driver!(config, "driver_idrisi");
    handle_gdal_driver!(config, "driver_pds");
    handle_gdal_driver!(config, "driver_sdts");
    handle_gdal_driver!(config, "driver_vrt");
    handle_gdal_driver!(config, "driver_mem");

    if cfg!(feature = "driver_sqlite") {
        let sqlite3_include_dir =
            std::env::var("DEP_SQLITE3_INCLUDE").expect("This is set by libsqlite3-sys");
        let sqlite3_lib_dir = std::env::var("DEP_SQLITE3_LIB_DIR").expect("set by libsqlite3-sys");
        let mut sqlite3_lib = PathBuf::from(sqlite3_lib_dir);
        sqlite3_lib.push("libsqlite3.a");
        let sqlite3_include_dir = PathBuf::from(sqlite3_include_dir);

        config
            .define("GDAL_USE_SQLITE3", "ON")
            .define("SQLite3_INCLUDE_DIR", print_path(&sqlite3_include_dir))
            .define("SQLite3_LIBRARY", print_path(&sqlite3_lib))
            .define("OGR_ENABLE_DRIVER_SQLITE", "ON");
    } else {
        config.define("GDAL_USE_SQLITE3", "OFF");
    }
    // these drivers depend on sqlite
    handle_ogr_driver!(config, "driver_gpkg");
    handle_ogr_driver!(config, "driver_vfk");

    if cfg!(feature = "driver_hdf5") {
        let hdf5_dir = std::env::var("DEP_HDF5_ROOT").expect("This is set by hdf5-sys");
        let hdf5_lib = std::env::var("DEP_HDF5_LIBRARY").expect("This is set by hdf5-sys");
        let hdf5_lib_dir = find_library(&hdf5_lib, &hdf5_dir);
        let mut hdf5_cc = PathBuf::from(&hdf5_dir);
        hdf5_cc.push("bin");
        hdf5_cc.push("h5cc");
        let hdf5_include = std::env::var("DEP_HDF5_INCLUDE").expect("This is set by hdf5-sys");
        let hdf5_include = PathBuf::from(&hdf5_include);
        config
            .define("GDAL_USE_HDF5", "ON")
            .define("HDF5_C_COMPILER_EXECUTABLE", print_path(&hdf5_cc))
            .define("HDF5_C_INCLUDE_DIR", print_path(&hdf5_include))
            .define("HDF5_hdf5_LIBRARY_DEBUG", print_path(&hdf5_lib_dir))
            .define("HDF5_hdf5_LIBRARY_RELEASE", print_path(&hdf5_lib_dir))
            .define("GDAL_ENABLE_DRIVER_HDF5", "ON")
            .define("HDF5_USE_STATIC_LIBRARIES", "ON");
    } else {
        config.define("GDAL_USE_HDF5", "OFF");
    }

    if cfg!(feature = "driver_netcdf") {
        let netcdf_include =
            std::env::var("DEP_NETCDF_INCLUDEDIR").expect("This is set by netcdf-sys");
        let netcdf_root = format!("{netcdf_include}/..");

        let netcdf_include = PathBuf::from(netcdf_include);
        let netcdf_root = PathBuf::from(netcdf_root);
        let netcdf_lib = find_library("netcdf", &netcdf_root);

        config
            .define("GDAL_USE_NETCDF", "ON")
            .define("NETCDF_INCLUDE_DIR", print_path(&netcdf_include))
            .define("NETCDF_LIBRARY", print_path(&netcdf_lib))
            .define("GDAL_ENABLE_DRIVER_NETCDF", "ON");
    } else {
        config.define("GDAL_USE_NETCDF", "OFF");
    }

    if cfg!(feature = "curl-sys") {
        let curl_root = std::env::var("DEP_CURL_ROOT").expect("set from curl-sys");
        let mut curl_include = PathBuf::from(&curl_root);
        curl_include.push("include");
        let mut curl_lib = PathBuf::from(curl_root);
        curl_lib.push("build");
        curl_lib.push("libcurl.a");
        config
            .define("GDAL_USE_CURL", "ON")
            .define("CURL_INCLUDE_DIR", print_path(&curl_include))
            .define("CURL_LIBRARY_DEBUG", print_path(&curl_lib))
            .define("CURL_LIBRARY_RELEASE", print_path(&curl_lib))
            .define("CURL_USE_STATIC_LIBS", "ON");
    } else {
        config.define("GDAL_USE_CURL", "OFF");
    }

    handle_ogr_driver!(config, "driver_amigocloud");
    handle_ogr_driver!(config, "driver_carto");
    handle_ogr_driver!(config, "driver_daas");
    handle_ogr_driver!(config, "driver_eeda");
    handle_ogr_driver!(config, "driver_elastic");
    handle_ogr_driver!(config, "driver_ngw");
    handle_gdal_driver!(config, "driver_ogcapi");
    handle_gdal_driver!(config, "driver_plmosaic");
    handle_gdal_driver!(config, "driver_wcs");
    handle_gdal_driver!(config, "driver_wms");
    handle_gdal_driver!(config, "driver_wmts");

    if cfg!(feature = "driver_pg") {
        let pq_include = std::env::var("DEP_PQ_SYS_SRC_INCLUDE").expect("this is set by pq-src");
        let pq_lib = std::env::var("DEP_PQ_SYS_SRC_LIB_DIR").expect("this is set by pq-src");
        let pq_lib_path = std::path::PathBuf::from(&pq_lib);
        println!("cargo:rustc-link-search=native={}", pq_lib_path.display());
        let pq_lib_path = if pq_lib_path.join("libpq.a").exists() {
            pq_lib_path.join("libpq.a")
        } else if pq_lib_path.join("pq.lib").exists() {
            pq_lib_path.join("pq.lib")
        } else {
            panic!("Libpq not found in {pq_lib}");
        };
        let pq_include = PathBuf::from(pq_include);
        println!("cargo:rustc-link-lib=static=pq");
        config
            .define("GDAL_USE_POSTGRESQL", "ON")
            .define("PostgreSQL_INCLUDE_DIR", print_path(&pq_include))
            .define("PostgreSQL_LIBRARY_DEBUG", print_path(&pq_lib_path))
            .define("PostgreSQL_LIBRARY_RELEASE", print_path(&pq_lib_path))
            .define("OGR_ENABLE_DRIVER_PG", "ON");
    } else {
        config.define("GDAL_USE_POSTGRESQL", "OFF");
    }
    handle_gdal_driver!(config, "driver_postgis_raster");

    if cfg!(feature = "geos") {
        config.define("GDAL_USE_GEOS", "ON");
    } else {
        config.define("GDAL_USE_GEOS", "OFF");
    }

    if cfg!(feature = "geos_static") {
        let geos_root = std::env::var("DEP_GEOSSRC_ROOT").expect("this is set by geos-src");
        let mut geos_include = PathBuf::from(&geos_root);
        geos_include.push("include");
        config.define("GEOS_INCLUDE_DIR", print_path(&geos_include));
        let lib_path = find_library("geos", geos_root);
        config.define("GEOS_LIBRARY", print_path(&lib_path));
    }

    if cfg!(target_env = "msvc") {
        // otherwise there are linking issues
        // because rust always links the
        // MSVC release runtime
        config.profile("Release");
        // see
        // https://github.com/OSGeo/PROJ/commit/6e9b324ab7bf5909df7e68409e060282db14fa54#diff-af8fe2f9d33a9c3408ff7683bfebd1e2334b4506f559add92406be3e150268fb
        config.cxxflag("-DPROJ_DLL=");
        // that windows library is somehow required
        println!("cargo:rustc-link-lib=Wbemuuid");
    }

    let res = config.build();

    // sometimes it's lib and sometimes it's lib64 and sometimes `build/lib`
    let lib_dir = res.join("lib64");
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_str().unwrap()
    );
    let lib_dir = res.join("lib");
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_str().unwrap()
    );
    let lib_dir = res.join("build").join("lib");
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_str().unwrap()
    );

    //gdal likes to create gdal_d when configured as debug and on MSVC, so link to that one if it exists
    if res.join("lib").join("gdald.lib").exists() {
        println!("cargo:rustc-link-lib=static=gdald");
    } else {
        println!("cargo:rustc-link-lib=static=gdal");
    }
}

// cmake sometimes does not like windows paths like `c:\\whatever\folder`
// it seems to tread `\` as escape sequence in some cases, therefore
// we rewrite the path here to always use `/` as separator
// https://github.com/OSGeo/gdal/issues/9935
fn print_path(path: &std::path::Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_str().unwrap())
        .collect::<Vec<_>>()
        .join("/")
}
