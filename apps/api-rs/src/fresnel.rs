pub struct FresnelResult {
    pub terrain_bulge: Vec<f64>,
    pub los_h: Vec<f64>,
    pub fresnel_r: Vec<f64>,
    pub obstructs_los: Vec<bool>,
    pub violates_f1: Vec<bool>,
    pub violates_f60: Vec<bool>,
}

pub fn fresnel_profile_analysis(
    distances: &[f64],
    elevations: &[f64],
    tx_antenna_h: f64,
    rx_antenna_h: f64,
    dist_m: f64,
    wavelength_m: f64,
    k_factor: f64,
) -> FresnelResult {
    let n = distances.len();
    let a_eff = k_factor * 6_371_000.0;

    let mut terrain_bulge = Vec::with_capacity(n);
    let mut los_h = Vec::with_capacity(n);
    let mut fresnel_r = Vec::with_capacity(n);
    let mut obstructs_los = Vec::with_capacity(n);
    let mut violates_f1 = Vec::with_capacity(n);
    let mut violates_f60 = Vec::with_capacity(n);

    for i in 0..n {
        let d = distances[i];
        let t = if dist_m > 0.0 { d / dist_m } else { 0.0 };
        let bulge = (d * (dist_m - d)) / (2.0 * a_eff);
        let tb = elevations[i] + bulge;
        let los = tx_antenna_h + t * (rx_antenna_h - tx_antenna_h);
        let d2 = dist_m - d;
        let fr = if d > 0.0 && d2 > 0.0 {
            (wavelength_m * d * d2 / (d + d2)).sqrt()
        } else {
            0.0
        };

        terrain_bulge.push(tb);
        los_h.push(los);
        fresnel_r.push(fr);
        obstructs_los.push(tb > los);
        violates_f1.push(tb > (los - fr));
        violates_f60.push(tb > (los - 0.6 * fr));
    }

    FresnelResult {
        terrain_bulge,
        los_h,
        fresnel_r,
        obstructs_los,
        violates_f1,
        violates_f60,
    }
}

pub fn apply_coverage_colors(
    prx_grid: &[Vec<f32>],
    thresholds: &[f64],
    colors: &[[u8; 4]],
    rows: usize,
    cols: usize,
) -> Vec<Vec<[u8; 4]>> {
    let n_thresh = thresholds.len();
    let mut rgba_out: Vec<Vec<[u8; 4]>> = (0..rows)
        .map(|_| (0..cols).map(|_| [0u8; 4]).collect())
        .collect();

    #[allow(clippy::needless_range_loop)]
    for i in 0..rows {
        let out_row = rows - 1 - i;
        for j in 0..cols {
            let v = prx_grid[i][j];
            if v.is_nan() {
                rgba_out[out_row][j] = [0, 0, 0, 0];
                continue;
            }
            let mut k = n_thresh;
            #[allow(clippy::needless_range_loop)]
            for t_idx in 0..n_thresh {
                if v as f64 >= thresholds[t_idx] {
                    k = t_idx;
                    break;
                }
            }
            rgba_out[out_row][j] = colors[k];
        }
    }
    rgba_out
}
