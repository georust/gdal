use crate::spatial_ref::SpatialRef;
use gdal_sys::OGRwkbGeometryType;

/// Parameters for [`Dataset::create_layer`].
#[derive(Clone, Debug)]
pub struct LayerOptions<'a> {
    /// The name of the newly created layer. May be an empty string.
    pub name: &'a str,
    /// The SRS of the newly created layer, or `None` for no SRS.
    pub srs: Option<&'a SpatialRef>,
    /// The type of geometry for the new layer.
    pub ty: OGRwkbGeometryType::Type,
    /// Additional driver-specific options to pass to GDAL, in the form `name=value`.
    pub options: Option<&'a [&'a str]>,
}

const EMPTY_LAYER_NAME: &str = "";

impl<'a> Default for LayerOptions<'a> {
    /// Returns creation options for a new layer with no name, no SRS and unknown geometry type.
    fn default() -> Self {
        LayerOptions {
            name: EMPTY_LAYER_NAME,
            srs: None,
            ty: OGRwkbGeometryType::wkbUnknown,
            options: None,
        }
    }
}
