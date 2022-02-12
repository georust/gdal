mod utils;

use gdal::vector::LayerAccess;
use gdal::Dataset;

fn main() {
    let ds = Dataset::open(fixture!("roads.geojson")).unwrap();
    let mut layer = ds.layer(0).unwrap();
    for _ in layer.features() {
        let _ = layer.defn();
    }
}
