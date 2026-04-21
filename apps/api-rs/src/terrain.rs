const R: f64 = 6_371_000.0;

pub fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dphi = (lat2 - lat1).to_radians();
    let dlambda = (lon2 - lon1).to_radians();
    let a = (dphi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (dlambda / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().asin()
}

pub fn bearing_destination(lat: f64, lon: f64, bearing_deg: f64, dist_m: f64) -> (f64, f64) {
    let brng = bearing_deg.to_radians();
    let lat_r = lat.to_radians();
    let lon_r = lon.to_radians();
    let d_r = dist_m / R;
    let lat2 = (lat_r.sin() * d_r.cos() + lat_r.cos() * d_r.sin() * brng.cos()).asin();
    let lon2 =
        lon_r + (brng.sin() * d_r.sin() * lat_r.cos()).atan2(d_r.cos() - lat_r.sin() * lat2.sin());
    (lat2.to_degrees(), lon2.to_degrees())
}

pub fn build_pfl(elevations: &[f64], step_m: f64) -> Vec<f64> {
    if elevations.is_empty() {
        return vec![];
    }
    let n = (elevations.len() - 1) as f64;
    let mut pfl = Vec::with_capacity(elevations.len() + 2);
    pfl.push(n);
    pfl.push(step_m);
    pfl.extend_from_slice(elevations);
    pfl
}

/// Interpolate NaN values in a slice using nearest-neighbor averaging.
/// Kept for potential future use in elevation processing.
#[allow(dead_code)]
pub fn interpolate_nans(values: &mut [f64]) {
    if values.is_empty() {
        return;
    }
    for i in 0..values.len() {
        if values[i].is_nan() {
            let left = values[..i].iter().rev().find(|v| !v.is_nan()).copied();
            let right = values[i + 1..].iter().find(|v| !v.is_nan()).copied();
            values[i] = match (left, right) {
                (Some(l), Some(r)) => (l + r) / 2.0,
                (Some(l), None) => l,
                (None, Some(r)) => r,
                (None, None) => f64::NAN,
            };
        }
    }
}

/// Compute the initial bearing (forward azimuth) from point 1 to point 2.
/// Kept for potential future use in bearing-based calculations.
#[allow(dead_code)]
pub fn initial_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let dlon = (lon2 - lon1).to_radians();
    let lat1r = lat1.to_radians();
    let lat2r = lat2.to_radians();
    let x = dlon.sin() * lat2r.cos();
    let y = lat1r.cos() * lat2r.sin() - lat1r.sin() * lat2r.cos() * dlon.cos();
    let brng = y.atan2(x).to_degrees();
    (brng + 360.0) % 360.0
}
