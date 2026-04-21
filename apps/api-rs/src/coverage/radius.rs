use crate::antenna::antenna_gain_factor;
use crate::itm_bridge::itm_p2p_loss;
use crate::models::CoverageRadiusResponse;
use crate::signal_levels::sample_line_from_grid;
use crate::terrain::{bearing_destination, build_pfl};

const RADIUS_CONSECUTIVE_MISS_LIMIT: usize = 3;

pub fn compute_coverage_radius(
    grid_data: &[Vec<f32>],
    grid_meta: &RadiusGridMeta,
    args: &RadiusWorkerArgs,
) -> CoverageRadiusResponse {
    let mut radius_per_bearing: Vec<(f64, f64)> = Vec::with_capacity(360);

    for bearing in 0..360 {
        let bearing_f = bearing as f64;
        let ant_gain = antenna_gain_factor(
            bearing_f,
            args.antenna_az_deg,
            args.antenna_beamwidth_deg,
            25.0,
        );
        let profile_step_target = (100.0_f64).max(args.sweep_step_m * 0.5);

        let mut consecutive_below = 0usize;
        let mut last_good = 0.0f64;
        let mut d = args.sweep_step_m;

        while d <= args.search_max_m {
            let (lat_end, lon_end) = bearing_destination(args.tx_lat, args.tx_lon, bearing_f, d);
            let n_pts = ((d / profile_step_target).round() as usize).clamp(3, 500);
            let step_m = d / (n_pts - 1) as f64;

            let elevs = sample_line_from_grid(
                grid_data,
                grid_meta.min_lat,
                grid_meta.max_lat,
                grid_meta.min_lon,
                grid_meta.max_lon,
                grid_meta.n_lat,
                grid_meta.n_lon,
                args.tx_lat,
                args.tx_lon,
                lat_end,
                lon_end,
                n_pts,
            );
            let elevs_f64: Vec<f64> = elevs.iter().map(|&v| v as f64).collect();
            let pfl = build_pfl(&elevs_f64, step_m);

            let res = itm_p2p_loss(
                args.tx_h_m,
                args.rx_h_m,
                &pfl,
                args.climate,
                args.n0,
                args.f_mhz,
                args.polarization,
                args.epsilon,
                args.sigma,
                12,
                args.time_pct,
                args.location_pct,
                args.situation_pct,
            );
            let loss_ok = res.loss_db.is_finite();

            if loss_ok {
                let prx = args.eirp_dbm + ant_gain + args.rx_gain_dbi - res.loss_db;
                if prx >= args.rx_sensitivity_dbm {
                    last_good = d;
                    consecutive_below = 0;
                } else {
                    consecutive_below += 1;
                }
            } else {
                consecutive_below += 1;
            }

            if consecutive_below >= RADIUS_CONSECUTIVE_MISS_LIMIT {
                break;
            }
            d += args.sweep_step_m;
        }
        radius_per_bearing.push((bearing_f, (last_good / 1000.0 * 100.0).round() / 100.0));
    }

    let radii: Vec<f64> = radius_per_bearing.iter().map(|&(_, r)| r).collect();
    let max_r = radii.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_r = radii.iter().cloned().fold(f64::INFINITY, f64::min);
    let avg_r = if !radii.is_empty() {
        radii.iter().sum::<f64>() / radii.len() as f64
    } else {
        0.0
    };

    CoverageRadiusResponse {
        max_radius_km: (max_r * 100.0).round() / 100.0,
        min_radius_km: (min_r * 100.0).round() / 100.0,
        avg_radius_km: (avg_r * 100.0).round() / 100.0,
        per_bearing: radius_per_bearing,
    }
}

pub struct RadiusGridMeta {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
    pub n_lat: usize,
    pub n_lon: usize,
}

pub struct RadiusWorkerArgs {
    pub tx_lat: f64,
    pub tx_lon: f64,
    pub tx_h_m: f64,
    pub rx_h_m: f64,
    pub f_mhz: f64,
    pub polarization: i32,
    pub climate: i32,
    pub n0: f64,
    pub epsilon: f64,
    pub sigma: f64,
    pub time_pct: f64,
    pub location_pct: f64,
    pub situation_pct: f64,
    pub eirp_dbm: f64,
    pub rx_gain_dbi: f64,
    pub rx_sensitivity_dbm: f64,
    pub antenna_az_deg: Option<f64>,
    pub antenna_beamwidth_deg: f64,
    pub sweep_step_m: f64,
    pub search_max_m: f64,
}
