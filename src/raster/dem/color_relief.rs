use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use crate::cpl::CslStringList;
use crate::errors;
use crate::raster::dem::options::common_dem_options;

use super::options::CommonOptions;

/// Configuration options for [`color_relief()`][super::color_relief()].
#[derive(Debug, Clone)]
pub struct ColorReliefOptions {
    common_options: CommonOptions,
    color_config: PathBuf,
    alpha: Option<bool>,
    color_matching_mode: ColorMatchingMode,
}

impl ColorReliefOptions {
    /// Create a DEM-color-relief options set.
    ///
    /// `color_config` is a required text file mapping elevations to color specifiers.
    /// Generally this means 4 columns per line: the elevation value, and the corresponding
    /// _Red_, _Green_, and _Blue_ component (between 0 and 255).
    /// The elevation value can be any floating point value, or the `nv` keyword for the no-data value.
    ///
    /// The elevation can also be expressed as a percentage:
    /// 0% being the minimum value found in the raster, 100% the maximum value.
    ///
    /// An extra column can be optionally added for the alpha component.
    /// If it is not specified, full opacity (255) is assumed.
    ///
    /// Various field separators are accepted: comma, tabulation, spaces, ':'.
    ///
    /// Common colors used by GRASS can also be specified by using their name,
    /// instead of the RGB triplet. The supported list is:
    /// _white_, _black_, _red_, _green_, _blue_, _yellow_, _magenta_, _cyan_, _aqua_, _grey/gray_,
    /// _orange_, _brown_, _purple/violet_ and _indigo_.
    ///
    /// Note: The syntax of the color configuration file is derived from the one supported by
    /// GRASS `r.colors` utility. ESRI HDR color table files (`.clr`) also match that syntax.
    /// The alpha component and the support of tab and comma as separators are GDAL specific extensions.
    ///
    /// # Example
    /// Here's an example `.clr` file showing a number of the features described above.
    ///
    /// ```text
    /// 2600  white
    /// 2000  235 220 175
    /// 50%   190 185 135
    /// 1000  240 250 150
    /// 100   50  180  50 128
    /// nv    0     0   0   0
    /// ```
    /// See: [gdaldem color-relief](https://gdal.org/programs/gdaldem.html#color-relief) for
    /// details.
    pub fn new<P: AsRef<Path>>(color_config: P) -> Self {
        Self {
            common_options: Default::default(),
            color_config: color_config.as_ref().to_path_buf(),
            alpha: None,
            color_matching_mode: Default::default(),
        }
    }

    common_dem_options!();

    /// Add an alpha channel to the output raster
    pub fn with_alpha(&mut self, state: bool) -> &mut Self {
        self.alpha = Some(state);
        self
    }

    /// Specify the color matching mode.
    ///
    /// See [`ColorMatchingMode`] for details.
    pub fn with_color_matching_mode(&mut self, mode: ColorMatchingMode) -> &mut Self {
        self.color_matching_mode = mode;
        self
    }

    pub(crate) fn color_config(&self) -> &Path {
        &self.color_config
    }

    /// Render relevant common options into [`CslStringList`] values, as compatible with
    /// [`gdal_sys::GDALDEMProcessing`].
    pub fn to_options_list(&self) -> errors::Result<CslStringList> {
        let mut opts = CslStringList::default();

        self.store_common_options_to(&mut opts)?;

        if self.alpha == Some(true) {
            opts.add_string("-alpha")?;
        }

        match self.color_matching_mode {
            ColorMatchingMode::ExactColorEntry => opts.add_string("-exact_color_entry")?,
            ColorMatchingMode::NearestColorEntry => opts.add_string("-nearest_color_entry")?,
            _ => {}
        }

        Ok(opts)
    }
}

/// Color relief color matching mode
#[derive(Debug, Clone, Copy, Default)]
pub enum ColorMatchingMode {
    /// Colors between the given elevation values are blended smoothly.
    ///
    /// This is the default.
    #[default]
    Blended,
    /// Use strict matching when searching in the color configuration file.
    ///
    /// If no matching color entry is found, the "0,0,0,0" RGBA quadruplet will be used.
    ExactColorEntry,
    /// Use the RGBA quadruplet corresponding to the closest entry in the color configuration file.
    NearestColorEntry,
}

#[cfg(test)]
mod tests {
    use crate::assert_near;
    use crate::cpl::CslStringList;
    use crate::errors::Result;
    use crate::raster::dem::color_relief;
    use crate::raster::StatisticsAll;
    use crate::test_utils::{fixture, InMemoryFixture};
    use crate::Dataset;

    use super::*;

    #[test]
    fn test_options() -> Result<()> {
        let mut proc = ColorReliefOptions::new("/dev/null");
        proc.with_input_band(2.try_into().unwrap())
            .with_compute_edges(true)
            .with_alpha(true)
            .with_color_matching_mode(ColorMatchingMode::NearestColorEntry)
            .with_output_format("GTiff")
            .with_additional_options("CPL_DEBUG=ON".parse()?);

        let expected: CslStringList =
            "-compute_edges -b 2 -of GTiff CPL_DEBUG=ON -alpha -nearest_color_entry".parse()?;
        assert_eq!(expected.to_string(), proc.to_options_list()?.to_string());

        Ok(())
    }

    #[test]
    fn test_color_relief() -> Result<()> {
        let ds = Dataset::open(fixture("dem-hills.tiff"))?;

        let mut opts = ColorReliefOptions::new(fixture("color-relief.clr"));
        opts.with_compute_edges(true);

        let output = InMemoryFixture::new("dem-hills-relief.tiff");
        let cr = color_relief(&ds, output.path(), &opts)?;

        // These numbers were generated by extracting the output from:
        //    gdaldem color-relief -compute_edges -alpha fixtures/dem-hills.tiff fixtures/color-relief.clr target/dest.tiff
        //    gdalinfo -stats target/dest.tiff
        let expected = [
            StatisticsAll {
                min: 0.0,
                max: 255.0,
                mean: 204.15542606827012,
                std_dev: 57.41999604472401,
            },
            StatisticsAll {
                min: 0.0,
                max: 255.0,
                mean: 221.0581177507783,
                std_dev: 33.229287115978394,
            },
            StatisticsAll {
                min: 0.0,
                max: 255.0,
                mean: 164.13047910295617,
                std_dev: 60.78580825073262,
            },
        ];
        for (i, e) in expected.iter().enumerate() {
            let stats = cr.rasterband(i + 1)?.get_statistics(true, false)?.unwrap();
            assert_near!(StatisticsAll, stats, e, epsilon = 1e-8);
        }

        Ok(())
    }
}
