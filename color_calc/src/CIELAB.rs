#![allow(non_snake_case)]

use crate::CIELCH::LChPoint;

use color_calc_core::conversion;
use color_calc_core::difference::{
    CIE94Params, CIEDE2000Params, CMCParams, calc_cie76, calc_cie94, calc_ciede2000, calc_cmc_lc,
};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct LabPoint {
    pub(crate) L: f64,
    pub(crate) a: f64,
    pub(crate) b: f64,
}

#[derive(Debug, Error)]
pub enum LabValidationError {
    #[error("'L' is too small. Min: 0")]
    LTooSmall,
    #[error("'L' is too big. Max: 100")]
    LTooBig,
}

impl LabPoint {
    pub fn new(L: f64, a: f64, b: f64) -> Result<Self, LabValidationError> {
        if L < 0.0 {
            return Err(LabValidationError::LTooSmall);
        }
        if L > 100.0 {
            return Err(LabValidationError::LTooBig);
        }

        Ok(Self { L, a, b })
    }

    pub fn L(&self) -> f64 {
        self.L
    }

    pub fn a(&self) -> f64 {
        self.a
    }

    pub fn b(&self) -> f64 {
        self.b
    }

    pub fn to_tuple(&self) -> (f64, f64, f64) {
        self.into()
    }

    pub fn to_LCh(&self) -> LChPoint {
        self.into()
    }

    pub fn calc_cie76(&self, sample: &LabPoint) -> f64 {
        calc_cie76(self.to_tuple(), sample.to_tuple())
    }

    pub fn calc_cie94(&self, sample: &LabPoint, parameters: CIE94Params) -> f64 {
        calc_cie94(self.to_tuple(), sample.to_tuple(), parameters)
    }

    pub fn calc_ciede2000(&self, sample: &LabPoint, parameters: CIEDE2000Params) -> f64 {
        calc_ciede2000(self.to_tuple(), sample.to_tuple(), parameters)
    }

    pub fn calc_cmc_lc(&self, sample: &LChPoint, parameters: CMCParams) -> f64 {
        calc_cmc_lc(self.to_tuple(), sample.to_tuple(), parameters)
    }
}

impl From<&LabPoint> for (f64, f64, f64) {
    fn from(lab: &LabPoint) -> Self {
        (lab.L, lab.a, lab.b)
    }
}

impl From<&LChPoint> for LabPoint {
    fn from(value: &LChPoint) -> Self {
        let (L, a, b) = conversion::lch_to_lab(value.L, value.C, value.h);
        Self { L, a, b }
    }
}
