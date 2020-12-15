#!/usr/bin/env python3

from osgeo import ogr, osr
import os.path

c_dir = os.path.dirname(os.path.realpath(__file__))


def create_n_layer_sqlite_ds(name: str, n_layers: int, n_features: int):
    driver_name = "SQLite"
    driver = ogr.GetDriverByName(driver_name)

    file_name = os.path.join(c_dir, name)
    if os.path.exists(file_name):
        os.remove(file_name)

    source = driver.CreateDataSource(file_name)

    srs = osr.SpatialReference()
    srs.ImportFromEPSG(4326)

    for nl in range(0, n_layers):
        # create a layer
        layer = source.CreateLayer("layer_{}".format(nl), srs, geom_type=ogr.wkbPoint)

        # create a field "id" in the layer
        id_field = ogr.FieldDefn("id", ogr.OFTInteger)
        layer.CreateField(id_field)

        for ni in range(0, n_features):
            # add a feature to the layer
            feature_def = layer.GetLayerDefn()
            feature = ogr.Feature(feature_def)

            # add a point to the feature
            point = ogr.Geometry(ogr.wkbPoint)
            point.AddPoint(x=47.0 + nl, y=-122.0 + nl)
            feature.SetGeometry(point)

            # add the feature to the layer
            layer.CreateFeature(feature)

            # add a field
            feature.SetField("id", nl)

    source.Destroy()


if __name__ == '__main__':
    create_n_layer_sqlite_ds(name="three_layer_ds.s3db", n_layers=3, n_features=3)
