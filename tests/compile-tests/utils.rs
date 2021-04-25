#[macro_export]
macro_rules! fixture {
    ($name:expr) => {
        std::path::Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures")
            .as_path()
            .join($name)
            .as_path()
    };
}
