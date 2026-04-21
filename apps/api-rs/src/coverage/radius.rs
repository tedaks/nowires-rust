use crate::antenna::{antenna_gain_factor, DEFAULT_FRONT_BACK_DB};
use crate::itm_bridge::{itm_p2p_loss, ITMParams};
use crate::models::CoverageRadiusResponse;
use crate::rounding::round2;
use crate::signal_levels::sample_line_from_grid;
use crate::terrain::{bearing_destination, build_pfl};
use rayon::prelude::*;

use super::workers::GridMeta;

/// Default number of consecutive "miss" samples before halting radius sweep
const RADIUS_CONSECUTIVE_MISS_LIMIT: usize = 3;
/// ITM mdvar parameter (radius mode)
const RADIUS_MDVAR: i32 = 12;
/// Minimum profile step target for radius sweep (meters)
const RADIUS_MIN_PROFILE_STEP_M: f64 = 100.0;

pub fn compute_coverage_radius(
    grid_data: &[Vec<f32>],
    grid_meta: &GridMeta,
    args: &RadiusWorkerArgs,
) -> CoverageRadiusResponse {
    let ITMParams {
        tx_h_m,
        rx_h_m,
        climate,
        n0,
        f_mhz,
        polarization,
        epsilon,
        sigma,
        time_pct,
        location_pct,
        situation_pct,
    } = args.params;

    let radius_per_bearing: Vec<(f64, f64)> = (0..360)
        .into_par_iter()
        .map(|bearing| {
            let bearing_f = bearing as f64;
            let ant_gain = antenna_gain_factor(
                bearing_f,
                args.antenna_az_deg,
                args.antenna_beamwidth_deg,
                DEFAULT_FRONT_BACK_DB,
            );
            let profile_step_target = (RADIUS_MIN_PROFILE_STEP_M).max(args.sweep_step_m * 0.5);

            let mut consecutive_below = 0usize;
            let mut last_good = 0.0f64;
            let mut d = args.sweep_step_m;

            while d <= args.search_max_m {
                let (lat_end, lon_end) =
                    bearing_destination(args.tx_lat, args.tx_lon, bearing_f, d);
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
                    tx_h_m,
                    rx_h_m,
                    &pfl,
                    climate,
                    n0,
                    f_mhz,
                    polarization,
                    epsilon,
                    sigma,
                    RADIUS_MDVAR,
                    time_pct,
                    location_pct,
                    situation_pct,
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
            (bearing_f, round2(last_good / 1000.0))
        })
        .collect();

    let radii: Vec<f64> = radius_per_bearing.iter().map(|&(_, r)| r).collect();
    let max_r = radii.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_r = radii.iter().cloned().fold(f64::INFINITY, f64::min);
    let avg_r = if !radii.is_empty() {
        radii.iter().sum::<f64>() / radii.len() as f64
    } else {
        0.0
    };

    CoverageRadiusResponse {
        max_radius_km: round2(max_r),
        min_radius_km: round2(min_r),
        avg_radius_km: round2(avg_r),
        per_bearing: radius_per_bearing,
    }
}

pub struct RadiusWorkerArgs {
    pub tx_lat: f64,
    pub tx_lon: f64,
    pub params: ITMParams,
    pub eirp_dbm: f64,
    pub rx_gain_dbi: f64,
    pub rx_sensitivity_dbm: f64,
    pub antenna_az_deg: Option<f64>,
    pub antenna_beamwidth_deg: f64,
    pub sweep_step_m: f64,
    pub search_max_m: f64,
}
