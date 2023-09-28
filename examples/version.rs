use gdal::version::VersionInfo;

/// So you can do `cargo run --example version` for bug reports. :-)
fn main() {
    let report = VersionInfo::version_report();
    println!("{report}");
}
