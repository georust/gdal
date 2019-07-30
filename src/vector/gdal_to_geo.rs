use gdal_sys::{self, OGRwkbGeometryType};
use geo_types;
use vector::Geometry;

impl From<Geometry> for geo_types::Geometry<f64> {
    fn from(geo: Geometry) -> geo_types::Geometry<f64> {
        let geometry_type = geo.geometry_type();

        let ring = |n: usize| {
            let ring = unsafe { geo._get_geometry(n) };
            match ring.into() {
                geo_types::Geometry::LineString(r) => r,
                _ => panic!("Expected to get a LineString"),
            }
        };

        match geometry_type {
            OGRwkbGeometryType::wkbPoint | OGRwkbGeometryType::wkbPoint25D => {
                let (x, y, _) = geo.get_point(0);
                geo_types::Geometry::Point(geo_types::Point(geo_types::Coordinate { x, y }))
            }
            OGRwkbGeometryType::wkbMultiPoint | OGRwkbGeometryType::wkbMultiPoint25D => {
                let point_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let coords = (0..point_count)
                    .map(|n| match unsafe { geo._get_geometry(n) }.into() {
                        geo_types::Geometry::Point(p) => p,
                        _ => panic!("Expected to get a Point"),
                    })
                    .collect();
                geo_types::Geometry::MultiPoint(geo_types::MultiPoint(coords))
            }
            OGRwkbGeometryType::wkbLineString | OGRwkbGeometryType::wkbLineString25D => {
                let coords = geo.get_point_vec()
                    .iter()
                    .map(|&(x, y, _)| geo_types::Coordinate { x, y })
                    .collect();
                geo_types::Geometry::LineString(geo_types::LineString(coords))
            }
            OGRwkbGeometryType::wkbMultiLineString | OGRwkbGeometryType::wkbMultiLineString25D => {
                let string_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let strings = (0..string_count)
                    .map(|n| match unsafe { geo._get_geometry(n) }.into() {
                        geo_types::Geometry::LineString(s) => s,
                        _ => panic!("Expected to get a LineString"),
                    })
                    .collect();
                geo_types::Geometry::MultiLineString(geo_types::MultiLineString(strings))
            }
            OGRwkbGeometryType::wkbPolygon | OGRwkbGeometryType::wkbPolygon25D => {
                let ring_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let outer = ring(0);
                let holes = (1..ring_count).map(ring).collect();
                geo_types::Geometry::Polygon(geo_types::Polygon::new(outer, holes))
            }
            OGRwkbGeometryType::wkbMultiPolygon | OGRwkbGeometryType::wkbMultiPolygon25D => {
                let string_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let strings = (0..string_count)
                    .map(|n| match unsafe { geo._get_geometry(n) }.into() {
                        geo_types::Geometry::Polygon(s) => s,
                        _ => panic!("Expected to get a Polygon"),
                    })
                    .collect();
                geo_types::Geometry::MultiPolygon(geo_types::MultiPolygon(strings))
            }
            OGRwkbGeometryType::wkbGeometryCollection | OGRwkbGeometryType::wkbGeometryCollection25D => {
                let item_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let geometry_list = (0..item_count)
                    .map(|n| unsafe { geo._get_geometry(n) }.into())
                    .collect();
                geo_types::Geometry::GeometryCollection(geo_types::GeometryCollection(
                    geometry_list,
                ))
            }
            _ => panic!("Unknown geometry type"),
        }
    }
}
