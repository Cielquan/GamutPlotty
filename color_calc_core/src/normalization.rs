#![allow(non_snake_case)]

pub fn normalize_angle(angle_in_degrees: f64) -> f64 {
    let mut normalized_angle = angle_in_degrees;
    // Bring to [0, 360)
    normalized_angle %= 360.0;
    // If negative, add 360
    if normalized_angle < 0.0 {
        normalized_angle += 360.0;
    }

    // Now bring to (-180, 180]
    if normalized_angle > 180.0 {
        normalized_angle -= 360.0;
    }
    normalized_angle
}
