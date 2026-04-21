/// Round to 1 decimal place
pub fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Round to 2 decimal places
pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Round to 3 decimal places
pub fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}
