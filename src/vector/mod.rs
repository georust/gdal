pub use vector::dataset::Dataset;
pub use vector::layer::{Layer, FeatureIterator};
pub use vector::feature::Feature;
pub use vector::geometry::{Geometry, OwnedGeometry, FeatureGeometry, ToGdal};

mod ogr;
pub mod dataset;
pub mod layer;
pub mod feature;
pub mod geometry;
pub mod geom;

#[cfg(test)]
mod tests;
