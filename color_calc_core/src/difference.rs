//! # Color Difference Formulas
//!
//! This module implements CIE 76, CIE 94, CIEDE2000, and CMC(l:c) formulas.
//! Implementations are based on https://en.wikipedia.org/wiki/Color_difference and linked resources.
//!
//! ## Input Formats
//! - `calc_cie76`, `calc_cie94`, `calc_ciede2000`: `(L, a, b)` tuples
//! - `calc_cmc_lc`: `(L, C, h)` tuples (h in degrees)

#![allow(non_snake_case)]

use crate::normalization::normalize_angle;

pub fn calc_cie76(target: (f64, f64, f64), sample: (f64, f64, f64)) -> f64 {
    //! `target` and `sample` tuple are expected to have the values in this order: (L, a, b).
    let (L1, a1, b1) = target;
    let (L2, a2, b2) = sample;

    ((L2 - L1).powi(2) + (a2 - a1).powi(2) + (b2 - b1).powi(2)).sqrt()
}

#[derive(Debug, Clone, Copy)]
pub struct CIE94Params {
    pub k_L: f64,
    pub k_C: f64,
    pub k_H: f64,
    pub K1: f64,
    pub K2: f64,
}

impl CIE94Params {
    pub fn textiles() -> Self {
        Self {
            k_L: 2.0,
            k_C: 1.0,
            k_H: 1.0,
            K1: 0.048,
            K2: 0.014,
        }
    }

    pub fn graphic_arts() -> Self {
        Self {
            k_L: 1.0,
            k_C: 1.0,
            k_H: 1.0,
            K1: 0.045,
            K2: 0.015,
        }
    }
}

pub fn calc_cie94(
    target: (f64, f64, f64),
    sample: (f64, f64, f64),
    parameters: CIE94Params,
) -> f64 {
    //! `target` and `sample` tuple are expected to have the values in this order: (L, a, b).
    let (L1, a1, b1) = target;
    let (L2, a2, b2) = sample;

    let CIE94Params {
        k_L,
        k_C,
        k_H,
        K1,
        K2,
    } = parameters;

    let delta_L = L1 - L2;

    let C1 = (a1.powi(2) + b1.powi(2)).sqrt();
    let C2 = (a2.powi(2) + b2.powi(2)).sqrt();
    let delta_C = C1 - C2;

    let delta_a = a1 - a2;
    let delta_b = b1 - b2;
    let delta_H = (delta_a.powi(2) + delta_b.powi(2) - delta_C.powi(2))
        .max(0.0)
        .sqrt();

    let S_L = 1.0;
    let S_C = 1.0 + K1 * C1;
    let S_H = 1.0 + K2 * C1;

    let term_l = (delta_L / (k_L * S_L)).powi(2);
    let term_c = (delta_C / (k_C * S_C)).powi(2);
    let term_h = (delta_H / (k_H * S_H)).powi(2);

    (term_l + term_c + term_h).sqrt()
}

#[derive(Debug, Clone, Copy)]
pub struct CIEDE2000Params {
    pub k_L: f64,
    pub k_C: f64,
    pub k_H: f64,
}

impl CIEDE2000Params {
    pub fn unity() -> Self {
        Self {
            k_L: 1.0,
            k_C: 1.0,
            k_H: 1.0,
        }
    }
}

fn calc_ciede2000_impl(
    target: (f64, f64, f64),
    sample: (f64, f64, f64),
    parameters: CIEDE2000Params,
) -> [f64; 14] {
    //! Impl after https://hajim.rochester.edu/ece/sites/gsharma/ciede2000/
    //! `target` and `sample` tuple are expected to have the values in this order: (L, a, b).
    let (L1, a1, b1) = target;
    let (L2, a2, b2) = sample;

    let CIEDE2000Params { k_L, k_C, k_H } = parameters;

    // Formula 2
    let C1 = (a1.powi(2) + b1.powi(2)).sqrt();
    // Formula 2
    let C2 = (a2.powi(2) + b2.powi(2)).sqrt();
    // Formula 3
    let C_bar = (C1 + C2) / 2.0;

    // Formula 4 (value tested)
    let G = 0.5 * (1.0 - (C_bar.powi(7) / (C_bar.powi(7) + 25.0_f64.powi(7))).sqrt());
    // Formula 5 (value tested)
    let a1_prime = (1.0 + G) * a1;
    // Formula 5 (value tested)
    let a2_prime = (1.0 + G) * a2;

    // Formula 6 (value tested)
    let C1_prime = (a1_prime.powi(2) + b1.powi(2)).sqrt();
    // Formula 6 (value tested)
    let C2_prime = (a2_prime.powi(2) + b2.powi(2)).sqrt();

    // Formula 7 (value tested)
    let h1_prime = b1.atan2(a1_prime).to_degrees().rem_euclid(360.0);
    // Formula 7 (value tested)
    let h2_prime = b2.atan2(a2_prime).to_degrees().rem_euclid(360.0);

    // Formula 8
    let delta_L_prime = L2 - L1;
    // Formula 9
    let delta_C_prime = C2_prime - C1_prime;

    // Formula 10
    let delta_h_prime = if C1_prime * C2_prime == 0.0 {
        0.0
    } else if (h2_prime - h1_prime).abs() <= 180.0 {
        // case 0 - both checks above are omitted
        h2_prime - h1_prime
    } else if (h2_prime - h1_prime) > 180.0 {
        // case 1
        (h2_prime - h1_prime) - 360.0
    } else if (h2_prime - h1_prime) < -180.0 {
        // case 2
        (h2_prime - h1_prime) + 360.0
    } else {
        panic!("Invalid state - delta_h_prime")
    };

    // Formula 11
    let delta_H_prime =
        2.0 * (C1_prime * C2_prime).sqrt() * (delta_h_prime / 2.0).to_radians().sin();

    // Formula 12
    let L_prime_bar = (L1 + L2) / 2.0;
    // Formula 13
    let C_prime_bar = (C1_prime + C2_prime) / 2.0;

    // Formula 14 (value tested)
    let h_prime_bar = if C1_prime * C2_prime == 0.0 {
        h1_prime + h2_prime
    } else if (h1_prime - h2_prime).abs() <= 180.0 {
        (h1_prime + h2_prime) / 2.0
    } else if (h1_prime + h2_prime) < 360.0 {
        (h1_prime + h2_prime + 360.0) / 2.0
    } else {
        (h1_prime + h2_prime - 360.0) / 2.0
    };

    // Formula 15 (value tested)
    let T = 1.0 - 0.17 * (h_prime_bar - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_prime_bar).to_radians().cos()
        + 0.32 * (3.0 * h_prime_bar + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_prime_bar - 63.0).to_radians().cos();

    // Formula 16
    let delta_theta = 30.0 * (-((h_prime_bar - 275.0) / 25.0).powi(2)).exp();

    // Formula 17
    let R_C = 2.0 * (C_prime_bar.powi(7) / (C_prime_bar.powi(7) + 25.0_f64.powi(7))).sqrt();

    // Formula 18 (value tested)
    let S_L = 1.0
        + ((0.015 * (L_prime_bar - 50.0).powi(2)) / (20.0 + (L_prime_bar - 50.0).powi(2)).sqrt());
    // Formula 19 (value tested)
    let S_C = 1.0 + 0.045 * C_prime_bar;
    // Formula 20 (value tested)
    let S_H = 1.0 + 0.015 * C_prime_bar * T;

    // Formula 21 (value tested)
    let R_T = -((2.0 * delta_theta).to_radians().sin()) * R_C;

    // Formula 22 (value tested)
    let term_l = delta_L_prime / (k_L * S_L);
    let term_c = delta_C_prime / (k_C * S_C);
    let term_h = delta_H_prime / (k_H * S_H);
    let de2000 = (term_l.powi(2) + term_c.powi(2) + term_h.powi(2) + R_T * term_c * term_h).sqrt();

    // Order is coupled with `TEST_VALUE_NAMES`
    [
        de2000,
        G,
        a1_prime,
        a2_prime,
        C1_prime,
        C2_prime,
        h1_prime,
        h2_prime,
        h_prime_bar,
        T,
        S_L,
        S_C,
        S_H,
        R_T,
    ]
}

pub fn calc_ciede2000(
    target: (f64, f64, f64),
    sample: (f64, f64, f64),
    parameters: CIEDE2000Params,
) -> f64 {
    calc_ciede2000_impl(target, sample, parameters)[0]
}

#[derive(Debug, Clone, Copy)]
pub struct CMCParams {
    pub l: f64,
    pub c: f64,
}

impl CMCParams {
    pub fn acceptability() -> Self {
        Self { l: 2.0, c: 1.0 }
    }

    pub fn imperceptibility() -> Self {
        Self { l: 1.0, c: 1.0 }
    }
}

pub fn calc_cmc_lc(target: (f64, f64, f64), sample: (f64, f64, f64), parameters: CMCParams) -> f64 {
    //! `target` and `sample` tuple are expected to have the values in this order: (L, C, h).
    //! `h` is expected to be in degrees.
    // h1 is in degrees
    let (L1, C1, h1) = target;
    // h2 is in degrees
    let (L2, C2, h2) = sample;

    let CMCParams { l, c } = parameters;

    let F = (C1.powi(4) / (C1.powi(4) + 1900.0)).sqrt();
    let T = if 164.0 <= h1 && h1 <= 345.0 {
        0.56 + (0.2 * (h1 + 168.0).to_radians().cos()).abs()
    } else {
        0.36 + (0.4 * (h1 + 35.0).to_radians().cos()).abs()
    };

    let S_L = if L1 < 16.0 {
        0.511
    } else {
        (0.040975 * L1) / (1.0 + (0.01765 * L1))
    };
    let S_C = ((0.0638 * C1) / (1.0 + (0.0131 * C1))) + 0.638;
    let S_H = S_C * (F * T + 1.0 - F);

    // If either chroma is near zero, dH is 0 (avoid NaN or weird math)
    let delta_H = if C1 < 1e-9 || C2 < 1e-9 {
        0.0
    } else {
        2.0 * (C1 * C2).sqrt() * (normalize_angle(h2 - h1) / 2.0).to_radians().sin()
    };

    let term_l = ((L2 - L1) / (l * S_L)).powi(2);
    let term_c = ((C2 - C1) / (c * S_C)).powi(2);
    let term_h = (delta_H / S_H).powi(2);

    (term_l + term_c + term_h).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    // Test data self generated
    #[case((  0.0001,  -0.0001,  -0.0001), (  0.0001,   -0.0001,   -0.0001),   0.0000)]
    #[case(( 50.0000,   0.0001,  -0.0001), ( 50.0000,    0.0001,   -0.0001),   0.0000)]
    #[case((100.0000,   0.0001,   0.0001), (100.0000,    0.0001,    0.0001),   0.0000)]
    #[case(( 50.0000, 100.0000, 100.0000), ( 50.0000,  100.0000,  100.0000),   0.0000)]
    #[case(( 50.0000,  10.0000,  10.0000), ( 50.0000,   10.0000,   11.0000),   1.0000)]
    #[case(( 50.0000,  10.0001,  10.0001), ( 50.0000,   10.0001,   11.0001),   1.0000)]
    #[case(( 50.0000,  10.0000,  10.0000), ( 50.0000,   10.0000,   10.9999),   0.9999)]
    #[case(( 50.0000,  10.0000,  10.0000), ( 50.0000,   11.0000,   11.0000),   1.4142)]
    #[case(( 50.0000,  10.0000,  10.0000), ( 50.0000,  -11.0000,   11.0000),  21.0238)]
    #[case(( 50.0000,  10.0000,  10.0000), ( 50.0000,   11.0000,  -11.0000),  21.0238)]
    #[case(( 50.0000,  10.0000,  10.0000), ( 50.0000,  -11.0000,  -11.0000),  29.6985)]
    #[case(( 50.0000, -10.0000,  10.0000), ( 50.0000,   11.0000,   11.0000),  21.0238)]
    #[case(( 50.0000,  10.0000, -10.0000), ( 50.0000,   11.0000,   11.0000),  21.0238)]
    #[case(( 50.0000, -10.0000, -10.0000), ( 50.0000,   11.0000,   11.0000),  29.6985)]
    #[case(( 50.1234,   0.5678,  26.9012), ( 50.9876,    0.5432,   26.1098),   1.1721)]
    #[case(( 50.1234,   0.5678,  26.9012), ( 51.9876,    1.5432,   27.1098),   2.1143)]
    #[case(( 50.1234,   0.5678,  26.9012), ( 51.9876,    3.5432,   28.1098),   3.7133)]
    #[case(( 50.1234,   0.5678,  26.9012), ( 51.9876,    5.5432,   30.1098),   6.2068)]
    #[case(( 50.1234,   0.5678,  26.9012), ( 50.9876,   10.5432,    0.1098),  28.6013)]
    #[case((100.0000, 150.0000, 150.0000), (  0.0000, -150.0000, -150.0000), 435.8899)]
    fn run_cie76_validations(
        #[case] target: (f64, f64, f64),
        #[case] sample: (f64, f64, f64),
        #[case] expected: f64,
    ) {
        let epsilon = 1e-4;

        let result = calc_cie76(target, sample);

        let diff = (result - expected).abs();
        assert!(
            diff < epsilon,
            "CIE76 calculation failed.\n\
            Target (L,a,b): ({:.4}, {:.4}, {:.4})\n\
            Sample (L,a,b): ({:.4}, {:.4}, {:.4})\n\
            Expected ΔE:    {}\n\
            Actual ΔE:      {}\n\
            Difference:     {}",
            target.0,
            target.1,
            target.2,
            sample.0,
            sample.1,
            sample.2,
            expected,
            result,
            diff
        );
    }

    #[rstest]
    // Test data from https://github.com/colour-science/colour/blob/c9cfa1f333452cc05e42f09eebd2f3dd935a5981/colour/difference/tests/test_delta_e.py
    #[case(( 48.99183622, -0.10561667, 400.65619925), ( 50.65907324,  -0.11671910, 402.82235718), CIE94Params::graphic_arts(),  1.671119130541200)]
    #[case((100.00000000, 21.57210357, 272.22819350), (100.00000000, 426.67945353,  72.39590835), CIE94Params::graphic_arts(), 83.779225500887094)]
    #[case((100.00000000, 21.57210357, 272.22819350), (100.00000000,  74.05216981, 276.45318193), CIE94Params::graphic_arts(), 10.053931954553839)]
    #[case((100.00000000, 21.57210357, 272.22819350), (100.00000000, 426.67945353,  72.39590835), CIE94Params::textiles(),     88.335553057506502)]
    #[case((100.00000000, 21.57210357, 272.22819350), (100.00000000,  74.05216981, 276.45318193), CIE94Params::textiles(),     10.612657890048272)]
    #[case((100.00000000, 21.57210357, 272.22819350), (100.00000000,   8.32281957, -73.58297716), CIE94Params::textiles(),     60.368687261063329)]
    fn run_cie94_validations(
        #[case] target: (f64, f64, f64),
        #[case] sample: (f64, f64, f64),
        #[case] param: CIE94Params,
        #[case] expected: f64,
    ) {
        let epsilon = 1e-7;

        let result = calc_cie94(target, sample, param);

        let diff = (result - expected).abs();
        assert!(
            diff < epsilon,
            "CIE94 calculation failed.\n\
            Target (L,a,b): ({:.4}, {:.4}, {:.4})\n\
            Sample (L,a,b): ({:.4}, {:.4}, {:.4})\n\
            Param:          {:?}\n\
            Expected ΔE:    {}\n\
            Actual ΔE:      {}\n\
            Difference:     {}",
            target.0,
            target.1,
            target.2,
            sample.0,
            sample.1,
            sample.2,
            param,
            expected,
            result,
            diff
        );
    }

    fn _print_de2000_impl_validations_failure_table(
        target: (f64, f64, f64),
        sample: (f64, f64, f64),
        errors: &[(usize, f64, f64, f64)],
    ) {
        let name_col_width = TEST_VALUE_NAMES
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(4)
            .max(4);

        println!("\n");
        println!("❌ CIEDE2000 Validation Failed");
        println!("   Target: {:?}, Sample: {:?}", target, sample);

        let top_border = format!(
            "┌{}┬─────────────┬─────────────┬──────────┐",
            "─".repeat(name_col_width + 2),
        );

        let mid_border = format!(
            "├{}┼─────────────┼─────────────┼──────────┤",
            "─".repeat(name_col_width + 2),
        );

        let bot_border = format!(
            "└{}┴─────────────┴─────────────┴──────────┘",
            "─".repeat(name_col_width + 2),
        );

        println!("{}", top_border);
        println!(
            "│ {:<name_col_width$} │    Expected │         Got │     Diff │",
            "Name",
        );
        println!("{}", mid_border);

        for (idx, exp, got, diff) in errors {
            println!(
                "│ {:<name_col_width$} │ {:>11.4} │ {:>11.4} │ {:>8.6} │",
                TEST_VALUE_NAMES[*idx], exp, got, diff
            );
        }

        println!("{}", bot_border);
    }

    const TEST_VALUE_NAMES: [&str; 14] = [
        "de2000",
        "G",
        "a1_prime",
        "a2_prime",
        "C1_prime",
        "C2_prime",
        "h1_prime",
        "h2_prime",
        "h_prime_bar",
        "T",
        "S_L",
        "S_C",
        "S_H",
        "R_T",
    ];

    #[rstest]
    // Test data from: https://hajim.rochester.edu/ece/sites/gsharma/ciede2000/
    // Note, that test cases 21-24 have longer a2 and b2 values, which influences intermediate results.
    // Therefore below test data was adjusted accordingly for 4 decimals.
    #[case((50.0000,   2.6772, -79.7751), (50.0000,   0.0000, -82.7485), [ 2.0425, 0.0001,   2.6774,   0.0000, 79.8200, 82.7485, 271.9222, 270.0000, 270.9611, 0.6907, 1.0000, 4.6578, 1.8421, -1.7042 ])]
    #[case((50.0000,   3.1571, -77.2803), (50.0000,   0.0000, -82.7485), [ 2.8615, 0.0001,   3.1573,   0.0000, 77.3448, 82.7485, 272.3395, 270.0000, 271.1698, 0.6843, 1.0000, 4.6021, 1.8216, -1.7070 ])]
    #[case((50.0000,   2.8361, -74.0200), (50.0000,   0.0000, -82.7485), [ 3.4412, 0.0001,   2.8363,   0.0000, 74.0743, 82.7485, 272.1944, 270.0000, 271.0972, 0.6865, 1.0000, 4.5285, 1.8074, -1.7060 ])]
    #[case((50.0000,  -1.3802, -84.2814), (50.0000,   0.0000, -82.7485), [ 1.0000, 0.0001,  -1.3803,   0.0000, 84.2927, 82.7485, 269.0618, 270.0000, 269.5309, 0.7357, 1.0000, 4.7584, 1.9217, -1.6809 ])]
    #[case((50.0000,  -1.1848, -84.8006), (50.0000,   0.0000, -82.7485), [ 1.0000, 0.0001,  -1.1849,   0.0000, 84.8089, 82.7485, 269.1995, 270.0000, 269.5997, 0.7335, 1.0000, 4.7700, 1.9218, -1.6822 ])]
    #[case((50.0000,  -0.9009, -85.5211), (50.0000,   0.0000, -82.7485), [ 1.0000, 0.0001,  -0.9009,   0.0000, 85.5258, 82.7485, 269.3964, 270.0000, 269.6982, 0.7303, 1.0000, 4.7862, 1.9217, -1.6840 ])]
    #[case((50.0000,   0.0000,   0.0000), (50.0000,  -1.0000,   2.0000), [ 2.3669, 0.5000,   0.0000,  -1.5000,  0.0000,  2.5000,   0.0000, 126.8697, 126.8697, 1.2200, 1.0000, 1.0562, 1.0229,  0.0000 ])]
    #[case((50.0000,  -1.0000,   2.0000), (50.0000,   0.0000,   0.0000), [ 2.3669, 0.5000,  -1.5000,   0.0000,  2.5000,  0.0000, 126.8697,   0.0000, 126.8697, 1.2200, 1.0000, 1.0562, 1.0229,  0.0000 ])]
    #[case((50.0000,   2.4900,  -0.0010), (50.0000,  -2.4900,   0.0009), [ 7.1792, 0.4998,   3.7346,  -3.7346,  3.7346,  3.7346, 359.9847, 179.9862, 269.9854, 0.7212, 1.0000, 1.1681, 1.0404, -0.0022 ])]
    #[case((50.0000,   2.4900,  -0.0010), (50.0000,  -2.4900,   0.0010), [ 7.1792, 0.4998,   3.7346,  -3.7346,  3.7346,  3.7346, 359.9847, 179.9847, 269.9847, 0.7212, 1.0000, 1.1681, 1.0404, -0.0022 ])]
    #[case((50.0000,   2.4900,  -0.0010), (50.0000,  -2.4900,   0.0011), [ 7.2195, 0.4998,   3.7346,  -3.7346,  3.7346,  3.7346, 359.9847, 179.9831,  89.9839, 0.6175, 1.0000, 1.1681, 1.0346,  0.0000 ])]
    #[case((50.0000,   2.4900,  -0.0010), (50.0000,  -2.4900,   0.0012), [ 7.2195, 0.4998,   3.7346,  -3.7346,  3.7346,  3.7346, 359.9847, 179.9816,  89.9831, 0.6175, 1.0000, 1.1681, 1.0346,  0.0000 ])]
    #[case((50.0000,  -0.0010,   2.4900), (50.0000,   0.0009,  -2.4900), [ 4.8045, 0.4998,  -0.0015,   0.0013,  2.4900,  2.4900,  90.0345, 270.0311, 180.0328, 0.9779, 1.0000, 1.1121, 1.0365,  0.0000 ])]
    #[case((50.0000,  -0.0010,   2.4900), (50.0000,   0.0010,  -2.4900), [ 4.8045, 0.4998,  -0.0015,   0.0015,  2.4900,  2.4900,  90.0345, 270.0345, 180.0345, 0.9779, 1.0000, 1.1121, 1.0365,  0.0000 ])]
    #[case((50.0000,  -0.0010,   2.4900), (50.0000,   0.0011,  -2.4900), [ 4.7461, 0.4998,  -0.0015,   0.0016,  2.4900,  2.4900,  90.0345, 270.0380,   0.0362, 1.3197, 1.0000, 1.1121, 1.0493,  0.0000 ])]
    #[case((50.0000,   2.5000,   0.0000), (50.0000,   0.0000,  -2.5000), [ 4.3065, 0.4998,   3.7496,   0.0000,  3.7496,  2.5000,   0.0000, 270.0000, 315.0000, 0.8454, 1.0000, 1.1406, 1.0396, -0.0001 ])]
    #[case((50.0000,   2.5000,   0.0000), (73.0000,  25.0000, -18.0000), [27.1492, 0.3827,   3.4569,  34.5687,  3.4569, 38.9743,   0.0000, 332.4939, 346.2470, 1.4453, 1.1608, 1.9547, 1.4599, -0.0003 ])]
    #[case((50.0000,   2.5000,   0.0000), (61.0000,  -5.0000,  29.0000), [22.8977, 0.3981,   3.4954,  -6.9907,  3.4954, 29.8307,   0.0000, 103.5532,  51.7766, 0.6447, 1.0640, 1.7498, 1.1612,  0.0000 ])]
    #[case((50.0000,   2.5000,   0.0000), (56.0000, -27.0000,  -3.0000), [31.9030, 0.4206,   3.5514, -38.3556,  3.5514, 38.4728,   0.0000, 184.4723, 272.2362, 0.6521, 1.0251, 1.9455, 1.2055, -0.8219 ])]
    #[case((50.0000,   2.5000,   0.0000), (58.0000,  24.0000,  15.0000), [19.4535, 0.4098,   3.5244,  33.8342,  3.5244, 37.0102,   0.0000,  23.9095,  11.9548, 1.1031, 1.0400, 1.9120, 1.3353,  0.0000 ])]
    #[case((50.0000,   2.5000,   0.0000), (50.0000,   3.1736,   0.5854), [ 1.0000, 0.4997,   3.7494,   4.7596,  3.7494,  4.7954,   0.0000,   7.0118,   3.5059, 1.2616, 1.0000, 1.1923, 1.0808,  0.0000 ])]
    #[case((50.0000,   2.5000,   0.0000), (50.0000,   3.2972,   0.0000), [ 1.0000, 0.4997,   3.7493,   4.9450,  3.7493,  4.9450,   0.0000,   0.0000,   0.0000, 1.3202, 1.0000, 1.1956, 1.0861,  0.0000 ])]
    #[case((50.0000,   2.5000,   0.0000), (50.0000,   1.8634,   0.5757), [ 1.0000, 0.4999,   3.7497,   2.7949,  3.7497,  2.8536,   0.0000,  11.6391,   5.8196, 1.2197, 1.0000, 1.1486, 1.0604,  0.0000 ])]
    #[case((50.0000,   2.5000,   0.0000), (50.0000,   3.2592,   0.3350), [ 1.0000, 0.4997,   3.7493,   4.8879,  3.7493,  4.8994,   0.0000,   3.9207,   1.9603, 1.2883, 1.0000, 1.1946, 1.0836,  0.0000 ])]
    #[case((60.2574, -34.0099,  36.2677), (60.4626, -34.1751,  39.4387), [ 1.2644, 0.0017, -34.0678, -34.2333, 49.7590, 52.2238, 133.2085, 130.9584, 132.0835, 1.3010, 1.1427, 3.2946, 1.9951,  0.0000 ])]
    #[case((63.0109, -31.0961,  -5.8663), (62.8187, -29.7946,  -4.0864), [ 1.2630, 0.0490, -32.6194, -31.2542, 33.1427, 31.5202, 190.1951, 187.4490, 188.8221, 0.9402, 1.1831, 2.4549, 1.4560,  0.0000 ])]
    #[case((61.2901,   3.7196,  -5.3901), (61.4292,   2.2480,  -4.9620), [ 1.8731, 0.4966,   5.5668,   3.3644,  7.7487,  5.9950, 315.9240, 304.1385, 310.0313, 0.6952, 1.1586, 1.3092, 1.0717, -0.0032 ])]
    #[case((35.0831, -44.1164,   3.7933), (35.0232, -40.0716,   1.5901), [ 1.8645, 0.0063, -44.3939, -40.3237, 44.5557, 40.3550, 175.1161, 177.7418, 176.4290, 1.0168, 1.2148, 2.9105, 1.6476,  0.0000 ])]
    #[case((22.7233,  20.0904, -46.6940), (23.0331,  14.9730, -42.5619), [ 2.0373, 0.0026,  20.1424,  15.0118, 50.8532, 45.1317, 293.3339, 289.4279, 291.3809, 0.3636, 1.4014, 3.1597, 1.2617, -1.2537 ])]
    #[case((36.4612,  47.8580,  18.3852), (36.2715,  50.5065,  21.2231), [ 1.4146, 0.0013,  47.9197,  50.5716, 51.3256, 54.8444,  20.9901,  22.7660,  21.8781, 0.9239, 1.1943, 3.3888, 1.7357,  0.0000 ])]
    #[case((90.8027,  -2.0831,   1.4410), (91.1528,  -1.6435,   0.0447), [ 1.4441, 0.4999,  -3.1245,  -2.4651,  3.4408,  2.4655, 155.2410, 178.9612, 167.1011, 1.1546, 1.6110, 1.1329, 1.0511,  0.0000 ])]
    #[case((90.9257,  -0.5406,  -0.9208), (88.6381,  -0.8985,  -0.7239), [ 1.5381, 0.5000,  -0.8109,  -1.3477,  1.2270,  1.5298, 228.6315, 208.2412, 218.4363, 1.3916, 1.5930, 1.0620, 1.0288,  0.0000 ])]
    #[case(( 6.7747,  -0.2908,  -2.4247), ( 5.8714,  -0.0985,  -2.2286), [ 0.6377, 0.4999,  -0.4362,  -0.1477,  2.4636,  2.2335, 259.8025, 266.2073, 263.0049, 0.9556, 1.6517, 1.1057, 1.0337, -0.0004 ])]
    #[case(( 2.0776,   0.0795,  -1.1350), ( 0.9033,  -0.0636,  -0.5514), [ 0.9082, 0.5000,   0.1192,  -0.0954,  1.1412,  0.5596, 275.9978, 260.1842, 268.0910, 0.7826, 1.7246, 1.0383, 1.0100,  0.0000 ])]
    fn run_de2000_impl_validations(
        #[case] target: (f64, f64, f64),
        #[case] sample: (f64, f64, f64),
        #[case] expected: [f64; 14],
    ) {
        let epsilon = 1e-4;

        let result = calc_ciede2000_impl(target, sample, CIEDE2000Params::unity());

        let mut errors: Vec<(usize, f64, f64, f64)> = Vec::new();
        for i in 0..TEST_VALUE_NAMES.len() {
            let diff = (result[i] - expected[i]).abs();
            if diff >= epsilon {
                errors.push((i, expected[i], result[i], diff));
            }
        }

        if !errors.is_empty() {
            _print_de2000_impl_validations_failure_table(target, sample, &errors);
            panic!("Validation failed for {} indices.", errors.len());
        }
    }

    #[rstest]
    // Test data from https://github.com/colour-science/colour/blob/c9cfa1f333452cc05e42f09eebd2f3dd935a5981/colour/difference/tests/test_delta_e.py
    // Orignal Lab data was converted to LCh
    #[case(( 48.99183622, 400.6562131707640, 90.01510369565824), ( 50.65907324, 402.82237408987080,  90.016601639686120), CMCParams::acceptability(),      0.89969998)]
    #[case((100.00000000, 273.0815720415939, 85.46919413100304), (100.00000000, 432.77768381754740,   9.629825133389737), CMCParams::acceptability(),    172.70477129)]
    #[case((100.00000000, 273.0815720415939, 85.46919413100304), (100.00000000, 286.19938094410820,  75.004493434326000), CMCParams::acceptability(),     20.59732717)]
    #[case((100.00000000, 273.0815720415939, 85.46919413100304), (100.00000000, 432.77768381754740,   9.629825133389737), CMCParams::imperceptibility(), 172.70477129)]
    #[case((100.00000000, 273.0815720415939, 85.46919413100304), (100.00000000, 286.19938094410820,  75.004493434326000), CMCParams::imperceptibility(),  20.59732717)]
    #[case((100.00000000, 273.0815720415939, 85.46919413100304), (100.00000000,  74.05216980834429, 276.453181926213900), CMCParams::imperceptibility(), 121.71841479)]
    fn run_cmc_lc_validations(
        #[case] target: (f64, f64, f64),
        #[case] sample: (f64, f64, f64),
        #[case] param: CMCParams,
        #[case] expected: f64,
    ) {
        let epsilon = 1e-7;

        let result = calc_cmc_lc(target, sample, param);

        let diff = (result - expected).abs();
        assert!(
            diff < epsilon,
            "CMC l:c calculation failed.\n\
            Target (L,a,b): ({:.4}, {:.4}, {:.4})\n\
            Sample (L,a,b): ({:.4}, {:.4}, {:.4})\n\
            Param:          {:?}\n\
            Expected ΔE:    {}\n\
            Actual ΔE:      {}\n\
            Difference:     {}",
            target.0,
            target.1,
            target.2,
            sample.0,
            sample.1,
            sample.2,
            param,
            expected,
            result,
            diff
        );
    }
}
