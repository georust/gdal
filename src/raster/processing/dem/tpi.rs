#[cfg(test)]
mod tests {
    use crate::assert_near;
    use crate::errors::Result;
    use crate::raster::processing::dem::{DemCommonOptions, DemProcessing};
    use crate::raster::StatisticsAll;
    use crate::test_utils::{fixture, target};
    use crate::Dataset;

    #[test]
    fn tpi() -> Result<()> {
        let opts = DemCommonOptions::new();

        let ds = Dataset::open(fixture("dem-hills.tiff"))?;

        let slope = ds.topographic_position_index(target("dem-hills-slope.tiff"), &opts)?;

        let stats = slope.rasterband(1)?.get_statistics(true, false)?.unwrap();

        // These numbers were generated by extracting the output from:
        //    gdaldem tpi fixtures/dem-hills.tiff target/dest.tiff
        //    gdalinfo -stats target/dest.tiff
        let expected = StatisticsAll {
            min: -4.7376708984375,
            max: 4.7724151611328,
            mean: 0.00012131847966826,
            std_dev: 0.48943078832474,
        };

        assert_near!(StatisticsAll, stats, expected, epsilon = 1e-10);
        Ok(())
    }
}
