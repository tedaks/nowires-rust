/// Default front-to-back ratio (dB) for directional antenna pattern
pub const DEFAULT_FRONT_BACK_DB: f64 = 25.0;

pub fn antenna_gain_factor(
    bearing_from_tx_deg: f64,
    az_deg: Option<f64>,
    beamwidth_deg: f64,
    front_back_db: f64,
) -> f64 {
    let az = match az_deg {
        Some(a) => a,
        None => return 0.0,
    };
    let diff = ((bearing_from_tx_deg - az + 540.0) % 360.0) - 180.0;
    if diff.abs() <= beamwidth_deg / 2.0 {
        let x = diff / (beamwidth_deg / 2.0);
        -3.0 * x * x
    } else {
        -front_back_db
    }
}
