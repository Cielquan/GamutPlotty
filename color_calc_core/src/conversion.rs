#![allow(non_snake_case)]

pub fn lab_to_lch(L: f64, a: f64, b: f64) -> (f64, f64, f64) {
    //! `h` will be set to 0 if either both `a` and `b` are 0, or if `c` is smaller than 1e-6.
    let c = (a.powi(2) + b.powi(2)).sqrt();

    let h = if c < 1e-6 {
        0.0
    } else {
        b.atan2(a).to_degrees().rem_euclid(360.0)
    };

    (L, c, h)
}

pub fn lch_to_lab(L: f64, C: f64, h: f64) -> (f64, f64, f64) {
    let a = C * h.cos();
    let b = C * h.sin();

    (L, a, b)
}
