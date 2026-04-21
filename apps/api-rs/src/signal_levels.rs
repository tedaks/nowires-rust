pub struct SignalLevel {
    pub threshold_dbm: f64,
    pub rgba: [u8; 4],
    pub label: &'static str,
}

pub const SIGNAL_LEVELS: [SignalLevel; 6] = [
    SignalLevel {
        threshold_dbm: -60.0,
        rgba: [0, 110, 40, 210],
        label: "Excellent",
    },
    SignalLevel {
        threshold_dbm: -75.0,
        rgba: [0, 180, 80, 200],
        label: "Good",
    },
    SignalLevel {
        threshold_dbm: -85.0,
        rgba: [180, 220, 40, 195],
        label: "Fair",
    },
    SignalLevel {
        threshold_dbm: -95.0,
        rgba: [240, 180, 40, 190],
        label: "Marginal",
    },
    SignalLevel {
        threshold_dbm: -105.0,
        rgba: [230, 110, 40, 185],
        label: "Weak",
    },
    SignalLevel {
        threshold_dbm: -120.0,
        rgba: [200, 40, 40, 0],
        label: "No service",
    },
];

pub const THRESHOLDS: [f64; 6] = [-60.0, -75.0, -85.0, -95.0, -105.0, -120.0];

pub const COLORS: [[u8; 4]; 7] = [
    [0, 110, 40, 210],
    [0, 180, 80, 200],
    [180, 220, 40, 195],
    [240, 180, 40, 190],
    [230, 110, 40, 185],
    [200, 40, 40, 0],
    [90, 20, 20, 0],
];

#[allow(dead_code)]
pub fn prx_to_color(prx_dbm: f64) -> [u8; 4] {
    if !prx_dbm.is_finite() {
        return [0, 0, 0, 0];
    }
    for level in &SIGNAL_LEVELS {
        if prx_dbm >= level.threshold_dbm {
            return level.rgba;
        }
    }
    [90, 20, 20, 0]
}

#[allow(clippy::too_many_arguments)]
pub fn sample_line_from_grid(
    gd: &[Vec<f32>],
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
    n_lat: usize,
    n_lon: usize,
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
    n_pts: usize,
) -> Vec<f32> {
    if n_lat <= 1 || n_lon <= 1 {
        return vec![0.0f32; n_pts];
    }
    let d_lat = (max_lat - min_lat) / (n_lat - 1) as f64;
    let d_lon = (max_lon - min_lon) / (n_lon - 1) as f64;

    let mut result = Vec::with_capacity(n_pts);
    for k in 0..n_pts {
        let t = k as f64 / (n_pts - 1) as f64;
        let lat = lat1 + t * (lat2 - lat1);
        let lon = lon1 + t * (lon2 - lon1);

        let mut fy = (lat - min_lat) / d_lat;
        let mut fx = (lon - min_lon) / d_lon;
        fy = fy.clamp(0.0, n_lat as f64 - 1.0 - 1e-9);
        fx = fx.clamp(0.0, n_lon as f64 - 1.0 - 1e-9);

        let y0 = fy.floor() as usize;
        let x0 = fx.floor() as usize;
        let y1 = (y0 + 1).min(n_lat - 1);
        let x1 = (x0 + 1).min(n_lon - 1);
        let ty = fy - y0 as f64;
        let tx = fx - x0 as f64;

        let v00 = gd[y0][x0] as f64;
        let v01 = gd[y0][x1] as f64;
        let v10 = gd[y1][x0] as f64;
        let v11 = gd[y1][x1] as f64;
        let v = v00 * (1.0 - tx) * (1.0 - ty)
            + v01 * tx * (1.0 - ty)
            + v10 * (1.0 - tx) * ty
            + v11 * tx * ty;
        result.push(v as f32);
    }
    result
}
