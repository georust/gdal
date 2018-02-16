use vector::Geometry;
use geo;
use gdal_sys::{self, OGRwkbGeometryType};

impl geo::ToGeo<f64> for Geometry {
    fn to_geo(&self) -> geo::Geometry<f64> {
        let geometry_type = unsafe { gdal_sys::OGR_G_GetGeometryType(self.c_geometry()) };

        let ring = |n: usize| {
            let ring = unsafe { self._get_geometry(n) };
            return match ring.to_geo() {
                geo::Geometry::LineString(r) => r,
                _ => panic!("Expected to get a LineString")
            };
        };

        match geometry_type {
            OGRwkbGeometryType::wkbPoint => {
                let (x, y, _) = self.get_point(0);
                geo::Geometry::Point(geo::Point(geo::Coordinate{x: x, y: y}))
            },
            OGRwkbGeometryType::wkbMultiPoint => {
                let point_count = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
                let coords = (0..point_count)
                    .map(|n| {
                        match unsafe { self._get_geometry(n) }.to_geo() {
                            geo::Geometry::Point(p) => p,
                            _ => panic!("Expected to get a Point")
                        }
                    })
                    .collect();
                geo::Geometry::MultiPoint(geo::MultiPoint(coords))
            },
            OGRwkbGeometryType::wkbLineString => {
                let coords = self.get_point_vec().iter()
                    .map(|&(x, y, _)| geo::Point(geo::Coordinate{x: x, y: y}))
                    .collect();
                geo::Geometry::LineString(geo::LineString(coords))
            },
            OGRwkbGeometryType::wkbMultiLineString => {
                let string_count = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
                let strings = (0..string_count)
                    .map(|n| {
                        match unsafe { self._get_geometry(n) }.to_geo() {
                            geo::Geometry::LineString(s) => s,
                            _ => panic!("Expected to get a LineString")
                        }
                    })
                    .collect();
                geo::Geometry::MultiLineString(geo::MultiLineString(strings))
            },
            OGRwkbGeometryType::wkbPolygon => {
                let ring_count = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
                let outer = ring(0);
                let holes = (1..ring_count).map(|n| ring(n)).collect();
                geo::Geometry::Polygon(geo::Polygon::new(outer, holes))
            },
            OGRwkbGeometryType::wkbMultiPolygon => {
                let string_count = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
                let strings = (0..string_count)
                    .map(|n| {
                        match unsafe { self._get_geometry(n) }.to_geo() {
                            geo::Geometry::Polygon(s) => s,
                            _ => panic!("Expected to get a Polygon")
                        }
                    })
                    .collect();
                geo::Geometry::MultiPolygon(geo::MultiPolygon(strings))
            },
            OGRwkbGeometryType::wkbGeometryCollection => {
                let item_count = unsafe { gdal_sys::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
                let geometry_list = (0..item_count)
                    .map(|n| unsafe { self._get_geometry(n) }.to_geo())
                    .collect();
                geo::Geometry::GeometryCollection(geo::GeometryCollection(geometry_list))
            }
            _ => panic!("Unknown geometry type")
        }
    }
}
