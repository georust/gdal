pub use vector::dataset::Dataset;
pub use vector::layer::{Layer, FeatureIterator};
pub use vector::feature::Feature;
pub use vector::geometry::Geometry;

mod ogr;
pub mod dataset;
pub mod layer;
pub mod feature;
pub mod geometry;

#[cfg(test)]
mod tests;
