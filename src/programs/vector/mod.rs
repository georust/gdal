#[cfg(all(major_ge_3, minor_ge_1))]
mod vector_translate;

#[cfg(all(major_ge_3, minor_ge_1))]
pub use vector_translate::{
    vector_translate,
    VectorTranslateOptions
};