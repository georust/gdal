#[cfg(all(major_ge_3, minor_ge_1))]
mod mdimtranslate;
mod vrt;

#[cfg(all(major_ge_3, minor_ge_1))]
pub use mdimtranslate::{
    multi_dim_translate, MultiDimTranslateOptions,
};
pub use vrt::*;
