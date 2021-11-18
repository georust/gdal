use gdal::{cpl, Dataset, DatasetOptions, GdalOpenFlags};
use ndarray::ArrayD;

fn main() {
    let dataset_options = DatasetOptions {
        open_flags: GdalOpenFlags::GDAL_OF_MULTIDIM_RASTER,
        allowed_drivers: None,
        open_options: None,
        sibling_files: None,
    };
    let dataset = Dataset::open_ex("fixtures/byte_no_cf.nc", dataset_options).unwrap();
    let root_group = dataset.root_group().unwrap();
    let array_name = "Band1".to_string();
    let options = cpl::CslStringList::new();
    let md_array = root_group.md_array(array_name, options);
    let dimensions = md_array.get_dimensions().unwrap();
    let mut dimensions_size = Vec::new();
    for dimension in dimensions {
        dimensions_size.push(dimension.size());
    }
    let count = dimensions_size.clone();

    let array_start_index = vec![0, 0];
    let data: ArrayD<u8> = md_array
        .read_as_array(array_start_index, count, dimensions_size)
        .unwrap();
    println!("data: {:?}", data);
}
