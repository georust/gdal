//! Example of reading from OGR to a stream of Arrow array batches
//!
//! As of this writing (Jan 2024), there are two competing low-level Arrow libraries in Rust.
//! [`arrow`](https://github.com/apache/arrow-rs) is the "official" one, while
//! [`arrow2`](https://github.com/jorgecarleitao/arrow2) is a
//! [less active](https://github.com/jorgecarleitao/arrow2/issues/1429) alternative.
//!
//! Each library implements the same Arrow memory standard, and each implements the
//! ArrowArrayStream interface, so each can integrate with the GDAL `read_arrow_stream` API.
//!
//! This example will use `arrow`, but the process is
//! [similar](https://github.com/georust/gdal/blob/87497bf28509ea1b66b8e64000bd6b33fde0f31b/examples/read_ogr_arrow.rs#L23)
//! when using `arrow2`.

#[cfg(any(major_ge_4, all(major_is_3, minor_ge_6)))]
fn run() -> gdal::errors::Result<()> {
    use arrow::array::{Array as _, BinaryArray};
    use arrow::ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream};
    use arrow::record_batch::RecordBatchReader;
    use gdal::cpl::CslStringList;
    use gdal::vector::*;
    use gdal::Dataset;
    use std::path::Path;

    // Open a dataset and access a layer
    let dataset = Dataset::open(Path::new("fixtures/roads.geojson"))?;
    let mut layer = dataset.layer(0)?;

    // Instantiate an `ArrowArrayStream` for OGR to write into
    let mut output_stream = FFI_ArrowArrayStream::empty();

    // Take a pointer to it
    let output_stream_ptr = &mut output_stream as *mut FFI_ArrowArrayStream;

    // GDAL includes its own copy of the ArrowArrayStream struct definition. These are guaranteed
    // to be the same across implementations, but we need to manually cast between the two for Rust
    // to allow it.
    let gdal_pointer: *mut gdal::ArrowArrayStream = output_stream_ptr.cast();

    let mut options = CslStringList::new();
    options.set_name_value("INCLUDE_FID", "NO")?;

    // Read the layer's data into our provisioned pointer
    unsafe { layer.read_arrow_stream(gdal_pointer, &options)? }

    // The rest of this example is specific to the `arrow` crate.

    // `arrow` has a helper class `ArrowArrayStreamReader` to assist with iterating over the raw
    // batches
    let arrow_stream_reader = ArrowArrayStreamReader::try_new(output_stream).unwrap();

    // Get the index of the geometry column
    let geom_column_index = arrow_stream_reader
        .schema()
        .column_with_name("wkb_geometry")
        .unwrap()
        .0;

    // Iterate over the stream until it's finished
    for maybe_array in arrow_stream_reader {
        // Access the contained array
        let top_level_array = maybe_array.unwrap();

        // Get the geometry column
        let geom_column = top_level_array.column(geom_column_index);

        // Downcast it to a `BinaryArray`
        let binary_array = geom_column.as_any().downcast_ref::<BinaryArray>().unwrap();

        // Access the first row as WKB
        let _wkb_buffer = binary_array.value(0);

        println!("Number of geometries: {}", binary_array.len());
    }

    Ok(())
}

#[cfg(not(any(major_ge_4, all(major_is_3, minor_ge_6))))]
fn run() -> gdal::errors::Result<()> {
    Ok(())
}

fn main() -> gdal::errors::Result<()> {
    run()
}
