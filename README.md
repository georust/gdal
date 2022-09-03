# GDAL

[![Documentation](https://docs.rs/gdal/badge.svg)](https://docs.rs/gdal)
![Build Status](https://github.com/georust/gdal/workflows/CI/badge.svg)

[GDAL](http://gdal.org/) is a translator and processing library for various raster and vector geospatial data formats. 

This crate provides safe, idiomatic [Rust](http://www.rust-lang.org/) bindings for GDAL.

## Capabilities

GDAL is an incredibly powerful library. For a general understanding of its capabilities, a good place to get started is the [GDAL User-oriented documentation](https://gdal.org/user/index.html). These features include:

* Opening raster and vector file formats for reading/writing
* Translating between file formats
* Reading and writing metadata in raster and vector datasets
* Accessing raster bands and their metadata
* Reading and writing geospatial coordinate system and projection values
* Warping (resampling and re-projecting) between coordinate systems

## Documentation

This crate's [API documentation](https://docs.rs/crate/gdal) is hosted on [docs.rs](https://docs.rs). 

The Rust documentation is currently a work in progress, and may not cover requisite details on parameter semantics, value interpretation, etc. 
Therefore, the authoritative documentation is that of GDAL in the form of its [C](https://gdal.org/api/index.html#c-api) and [C++](https://gdal.org/api/index.html#id3) APIs.
The former is technically what this crate calls, but the latter is usually more clear and better documented.

## Usage

This crate provides high-level, idiomatic Rust bindings for GDAL.
To do that, it uses [`gdal-sys`](gdal-sys) internally, a low-level interface to the GDAL C library, which is generated using [`bindgen`](https://rust-lang.github.io/rust-bindgen/).
Using the `gdal-sys` crate directly is normally not needed, but it can be useful in order to call APIs that have not yet been exposed in `gdal`.

Building this crate assumes a compatible version of GDAL is installed with the corresponding header files and shared libraries.
This repository includes pre-generated bindings for GDAL 2.4 through 3.5 (see the`gdal-sys/prebuilt-bindings` directory).
If you're compiling against a later version of GDAL, you can enable the `bindgen` feature flag to have new bindings generated on the fly. 

## Community

This crate is part of the expansive (and expanding!) [`georust`](https://georust.org/) organization. Come join our discussions on [Discord](https://discord.gg/Fp2aape)!

## Contributing

This crate continues to evolve, and PRs are welcome. Make sure you are comfortable with the [Code of Conduct](CODE_OF_CONDUCT.md) and [License](LICENSE.txt) before submitting a PR.

## License

This library is released under the [MIT license](http://opensource.org/licenses/MIT)