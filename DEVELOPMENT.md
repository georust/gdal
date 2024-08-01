# Updating bundled gdal version for gdal-src

Perform the following steps:

```
git submodule init
git submodule update
cd gdal-src/source
git pull
git checkout v3.8.3 # corresponds to the tag you want to update to
cd ../../
git add gdal-src/source
git commit -m "Update bundled gdal version to 3.8.4"
```

These steps assume that there are no fundamental changes to the gdal build system.

