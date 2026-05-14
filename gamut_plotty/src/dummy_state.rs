use color_calc::CIELAB;

pub fn create_color_points() -> Vec<CIELAB::LabPoint> {
    vec![
        CIELAB::LabPoint::new(0.0001, -0.0001, -0.0001).unwrap(),
        CIELAB::LabPoint::new(50.0000, 0.0001, -0.0001).unwrap(),
        CIELAB::LabPoint::new(100.0000, 0.0001, 0.0001).unwrap(),
    ]
}
