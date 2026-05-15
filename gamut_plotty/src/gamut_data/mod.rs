mod cie_d50_10deg_1964;
mod cie_d50_10deg_2015;
mod cie_d50_2deg_1931;
mod cie_d50_2deg_2015;
mod cie_d65_10deg_1964;
mod cie_d65_10deg_2015;
mod cie_d65_2deg_1931;
mod cie_d65_2deg_2015;

use std::fmt;

pub use cie_d50_2deg_1931::GAMUT_BOUNDARY as D50_2DEG_1931;
pub use cie_d50_2deg_2015::GAMUT_BOUNDARY as D50_2DEG_2015;
pub use cie_d50_10deg_1964::GAMUT_BOUNDARY as D50_10DEG_1964;
pub use cie_d50_10deg_2015::GAMUT_BOUNDARY as D50_10DEG_2015;

pub use cie_d65_2deg_1931::GAMUT_BOUNDARY as D65_2DEG_1931;
pub use cie_d65_2deg_2015::GAMUT_BOUNDARY as D65_2DEG_2015;
pub use cie_d65_10deg_1964::GAMUT_BOUNDARY as D65_10DEG_1964;
pub use cie_d65_10deg_2015::GAMUT_BOUNDARY as D65_10DEG_2015;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Observer {
    #[default]
    CIE2deg1931,
    CIE2deg2015,
    CIE10deg1964,
    CIE10deg2015,
}

impl fmt::Display for Observer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Observer::CIE2deg1931 => write!(f, "CIE 2° (1931)"),
            Observer::CIE2deg2015 => write!(f, "CIE 2° (2015)"),
            Observer::CIE10deg1964 => write!(f, "CIE 10° (1964)"),
            Observer::CIE10deg2015 => write!(f, "CIE 10° (2015)"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Illuminant {
    D50,
    #[default]
    D65,
}

impl fmt::Display for Illuminant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Illuminant::D50 => write!(f, "D50"),
            Illuminant::D65 => write!(f, "D65"),
        }
    }
}

pub fn get_gamut_boundary_data(
    observer: Observer,
    illuminant: Illuminant,
) -> &'static [(f64, f64, f64)] {
    match (observer, illuminant) {
        (Observer::CIE2deg1931, Illuminant::D50) => &D50_2DEG_1931,
        (Observer::CIE2deg1931, Illuminant::D65) => &D65_2DEG_1931,
        (Observer::CIE2deg2015, Illuminant::D50) => &D50_2DEG_2015,
        (Observer::CIE2deg2015, Illuminant::D65) => &D65_2DEG_2015,
        (Observer::CIE10deg1964, Illuminant::D50) => &D50_10DEG_1964,
        (Observer::CIE10deg1964, Illuminant::D65) => &D65_10DEG_1964,
        (Observer::CIE10deg2015, Illuminant::D50) => &D50_10DEG_2015,
        (Observer::CIE10deg2015, Illuminant::D65) => &D65_10DEG_2015,
    }
}
