#![allow(non_snake_case)]

use crate::CIELAB::LabPoint;

use color_calc_core::conversion;
use color_calc_core::difference::{
    CIE94Params, CIEDE2000Params, CMCParams, calc_cie76, calc_cie94, calc_ciede2000, calc_cmc_lc,
};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct LChPoint {
    pub(crate) L: f64,
    pub(crate) C: f64,
    pub(crate) h: f64,
}

#[derive(Debug, Error)]
pub enum LChValidationError {
    #[error("'L' is too small. Min: 0")]
    LTooSmall,
    #[error("'L' is too big. Max: 100")]
    LTooBig,
    #[error("'C' is too small. Min: 0")]
    CTooSmall,
    #[error("'h' is too small. Min: 0")]
    HTooSmall,
    #[error("'h' is too big. Max: 100")]
    HTooBig,
}

impl LChPoint {
    pub fn new(L: f64, C: f64, h: f64) -> Result<Self, LChValidationError> {
        if L < 0.0 {
            return Err(LChValidationError::LTooSmall);
        }
        if L > 100.0 {
            return Err(LChValidationError::LTooBig);
        }

        if C < 0.0 {
            return Err(LChValidationError::CTooSmall);
        }

        if h < 0.0 {
            return Err(LChValidationError::HTooSmall);
        }
        if h > 360.0 {
            return Err(LChValidationError::HTooBig);
        }

        Ok(Self { L, C, h })
    }

    pub fn L(&self) -> f64 {
        self.L
    }

    pub fn C(&self) -> f64 {
        self.C
    }

    pub fn h(&self) -> f64 {
        self.h
    }

    pub fn to_tuple(&self) -> (f64, f64, f64) {
        self.into()
    }

    pub fn to_Lab(&self) -> LabPoint {
        self.into()
    }

    pub fn calc_cie76(&self, sample: &LChPoint) -> f64 {
        calc_cie76(LabPoint::from(sample).to_tuple(), sample.to_tuple())
    }

    pub fn calc_cie94(&self, sample: &LChPoint, parameters: CIE94Params) -> f64 {
        calc_cie94(
            LabPoint::from(sample).to_tuple(),
            sample.to_tuple(),
            parameters,
        )
    }

    pub fn calc_ciede2000(&self, sample: &LChPoint, parameters: CIEDE2000Params) -> f64 {
        calc_ciede2000(
            LabPoint::from(sample).to_tuple(),
            sample.to_tuple(),
            parameters,
        )
    }

    pub fn calc_cmc_lc(&self, sample: &LChPoint, parameters: CMCParams) -> f64 {
        calc_cmc_lc(self.to_tuple(), sample.to_tuple(), parameters)
    }
}

impl From<&LChPoint> for (f64, f64, f64) {
    fn from(lch: &LChPoint) -> Self {
        (lch.L, lch.C, lch.h)
    }
}

impl From<&LabPoint> for LChPoint {
    fn from(value: &LabPoint) -> Self {
        let (L, C, h) = conversion::lab_to_lch(value.L, value.a, value.b);
        Self { L, C, h }
    }
}
