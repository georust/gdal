use std::num::NonZeroUsize;

use crate::cpl::CslStringList;
use crate::errors;
use crate::raster::dem::options::common_dem_options;
use crate::raster::dem::DemSlopeAlg;

use super::options::CommonOptions;

/// Configuration options for [`hillshade()`][super::hillshade()].
#[derive(Debug, Clone, Default)]
pub struct HillshadeOptions {
    common_options: CommonOptions,
    algorithm: Option<DemSlopeAlg>,
    altitude: Option<f64>,
    azimuth: Option<f64>,
    scale: Option<f64>,
    shading: Option<ShadingMode>,
    z_factor: Option<f64>,
}

impl HillshadeOptions {
    /// Create a DEM-hillshade options set.
    pub fn new() -> Self {
        Default::default()
    }

    common_dem_options!();

    /// Specify the slope computation algorithm.
    pub fn with_algorithm(&mut self, algorithm: DemSlopeAlg) -> &mut Self {
        self.algorithm = Some(algorithm);
        self
    }

    /// Specify the altitude of the light, in degrees.
    ///
    /// `90` if the light comes from above the DEM, `0` if it is raking light.
    pub fn with_altitude(&mut self, altitude: f64) -> &mut Self {
        self.altitude = Some(altitude);
        self
    }

    /// Specify the azimuth of the light, in degrees:
    ///
    /// * `0` if it comes from the top of the raster,
    /// * `90` from the east,
    /// * etc.
    ///
    /// The default value, `315`, and should rarely be changed as it is the value generally
    /// used to generate shaded maps.
    pub fn with_azimuth(&mut self, azimuth: f64) -> &mut Self {
        self.azimuth = Some(azimuth);
        self
    }

    /// Apply a elevation scaling factor.
    ///
    /// Routine assumes x, y and z units are identical.
    /// If x (east-west) and y (north-south) units are identical, but z (elevation) units are different,
    /// this scale option can be used to set the ratio of vertical units to horizontal.
    ///
    /// For LatLong projections <u>near the equator</u>, where units of latitude and units of longitude are
    /// similar, elevation (z) units can be converted with the following values:
    ///
    /// * Elevation in feet: `370400`
    /// * Elevation in meters: `111120`
    ///
    /// For locations not near the equator, it would be best to reproject your raster first.
    pub fn with_scale(&mut self, scale: f64) -> &mut Self {
        self.scale = Some(scale);
        self
    }

    /// Specify the shading mode to render with.
    ///
    /// See [`ShadingMode`] for mode descriptions.
    pub fn with_shading_mode(&mut self, mode: ShadingMode) -> &mut Self {
        self.shading = Some(mode);
        self
    }

    /// Vertical exaggeration used to pre-multiply the elevations
    pub fn with_z_factor(&mut self, z_factor: f64) -> &mut Self {
        self.z_factor = Some(z_factor);
        self
    }

    /// Render relevant common options into [`CslStringList`] values, as compatible with
    /// [`gdal_sys::GDALDEMProcessing`].
    pub fn to_options_list(&self) -> errors::Result<CslStringList> {
        let mut opts = CslStringList::default();

        self.store_common_options_to(&mut opts)?;

        if let Some(alg) = self.algorithm {
            opts.add_string("-alg")?;
            opts.add_string(alg.to_gdal_option())?;
        }

        if let Some(scale) = self.scale {
            opts.add_string("-s")?;
            opts.add_string(&scale.to_string())?;
        }

        if let Some(mode) = self.shading {
            opts.add_string(mode.to_gdal_option())?;
        }

        if let Some(factor) = self.z_factor {
            opts.add_string("-z")?;
            opts.add_string(&factor.to_string())?;
        }

        if let Some(altitude) = self.altitude {
            opts.add_string("-alt")?;
            opts.add_string(&altitude.to_string())?;
        }

        if let Some(azimuth) = self.azimuth {
            opts.add_string("-az")?;
            opts.add_string(&azimuth.to_string())?;
        }

        Ok(opts)
    }
}

/// Hillshade shading mode.
#[derive(Debug, Clone, Copy)]
pub enum ShadingMode {
    /// Combination of slope and oblique shading.
    Combined,
    /// Multi-directional shading,
    ///
    /// A combination of hillshading illuminated from 225 deg, 270 deg, 315 deg, and 360 deg azimuth.
    ///
    /// See: <http://pubs.usgs.gov/of/1992/of92-422/of92-422.pdf>.
    Multidirectional,
    /// Shading which tries to minimize effects on other map features beneath.
    ///
    /// Can't be used `altitude` specification
    ///
    /// See: <http://maperitive.net/docs/Commands/GenerateReliefImageIgor.html>.
    Igor,
}

impl ShadingMode {
    fn to_gdal_option(self) -> &'static str {
        match self {
            ShadingMode::Combined => "-combined",
            ShadingMode::Multidirectional => "-multidirectional",
            ShadingMode::Igor => "-igor",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_near;
    use crate::cpl::CslStringList;
    use crate::errors::Result;
    use crate::raster::dem::hillshade;
    use crate::raster::StatisticsAll;
    use crate::test_utils::{fixture, InMemoryFixture};
    use crate::Dataset;

    use super::*;

    #[test]
    fn test_options() -> Result<()> {
        let mut proc = HillshadeOptions::new();
        proc.with_input_band(2.try_into().unwrap())
            .with_algorithm(DemSlopeAlg::ZevenbergenThorne)
            .with_scale(98473.0)
            .with_shading_mode(ShadingMode::Igor)
            .with_compute_edges(true)
            .with_azimuth(330.0)
            .with_altitude(45.0)
            .with_z_factor(2.0)
            .with_output_format("GTiff")
            .with_additional_options("CPL_DEBUG=ON".parse()?);

        let expected: CslStringList =
            "-compute_edges -b 2 -of GTiff CPL_DEBUG=ON -alg ZevenbergenThorne -s 98473 -igor -z 2 -alt 45 -az 330"
                .parse()?;
        assert_eq!(expected.to_string(), proc.to_options_list()?.to_string());

        Ok(())
    }

    #[test]
    fn test_hillshade() -> Result<()> {
        let ds = Dataset::open(fixture("dem-hills.tiff"))?;
        let scale_factor = 98473.2947;

        let mut opts = HillshadeOptions::new();
        opts.with_algorithm(DemSlopeAlg::ZevenbergenThorne)
            .with_shading_mode(ShadingMode::Igor)
            .with_z_factor(2.0)
            .with_scale(scale_factor);

        let output = InMemoryFixture::new("dem-hills-shade.tiff");
        let shade = hillshade(&ds, output.path(), &opts)?;

        let stats = shade.rasterband(1)?.get_statistics(true, false)?.unwrap();

        // These numbers were generated by extracting the output from:
        //    gdaldem hillshade -alg ZevenbergenThorne -s 98473.2947 -igor -z 2 fixtures/dem-hills.tiff target/dest.tiff
        //    gdalinfo -stats target/dest.tiff
        let expected = StatisticsAll {
            min: 128.0,
            max: 255.0,
            mean: 244.15731356401,
            std_dev: 16.76881437538,
        };

        assert_near!(StatisticsAll, stats, expected, epsilon = 1e-8);
        Ok(())
    }
}
