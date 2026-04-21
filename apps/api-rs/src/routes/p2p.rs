use crate::elevation::ElevationGrid;
use crate::fresnel::fresnel_profile_analysis;
use crate::itm_bridge::{itm_p2p_loss, PROP_MODE_NAMES};
use crate::models::*;
use crate::signal_levels::sample_line_from_grid;
use crate::terrain::{build_pfl, haversine_m};
use axum::Json;

const C: f64 = 299_792_458.0;

pub async fn p2p_handler(Json(req): Json<P2PRequest>) -> Json<P2PResponse> {
    let dist_m = haversine_m(req.tx.lat, req.tx.lon, req.rx.lat, req.rx.lon);
    let step_m = 30.0;
    let n_pts = ((dist_m / step_m).round() as usize).max(2);
    let n_pts = n_pts.min(5000);

    let elev = ElevationGrid::fetch(
        req.tx.lat.min(req.rx.lat) - 0.01,
        req.tx.lon.min(req.rx.lon) - 0.01,
        req.tx.lat.max(req.rx.lat) + 0.01,
        req.tx.lon.max(req.rx.lon) + 0.01,
        n_pts,
        "glo30",
    );

    let sample_n = n_pts.min(1000);
    let elevs = sample_line_from_grid(
        &elev.data,
        elev.min_lat,
        elev.max_lat,
        elev.min_lon,
        elev.max_lon,
        elev.n_lat,
        elev.n_lon,
        req.tx.lat,
        req.tx.lon,
        req.rx.lat,
        req.rx.lon,
        sample_n,
    );

    let elevs_f64: Vec<f64> = elevs
        .iter()
        .map(|&v| if v.is_nan() { 0.0 } else { v as f64 })
        .collect();
    let actual_step_m = dist_m / (sample_n - 1).max(1) as f64;

    let pfl = build_pfl(&elevs_f64, actual_step_m);

    let mdvar = 12;
    let result = itm_p2p_loss(
        req.tx.h_m,
        req.rx.h_m,
        &pfl,
        req.climate,
        req.n0,
        req.freq_mhz,
        req.polarization,
        req.epsilon,
        req.sigma,
        mdvar,
        req.time_pct,
        req.location_pct,
        req.situation_pct,
    );

    let tx_elev = elevs_f64[0];
    let rx_elev = elevs_f64[elevs_f64.len() - 1];
    let tx_antenna_h = tx_elev + req.tx.h_m;
    let rx_antenna_h = rx_elev + req.rx.h_m;

    let distances: Vec<f64> = (0..sample_n).map(|i| i as f64 * actual_step_m).collect();
    let wavelength_m = C / (req.freq_mhz * 1e6);

    let fresnel = fresnel_profile_analysis(
        &distances,
        &elevs_f64,
        tx_antenna_h,
        rx_antenna_h,
        dist_m,
        wavelength_m,
        req.k_factor,
    );

    let any_blockage = fresnel.obstructs_los.iter().any(|&b| b);
    let any_f1 = fresnel.violates_f1.iter().any(|&b| b);
    let any_f60 = fresnel.violates_f60.iter().any(|&b| b);

    let mut profile_data: Vec<ProfilePoint> = distances
        .iter()
        .enumerate()
        .map(|(i, &d)| ProfilePoint {
            d: (d * 10.0).round() / 10.0,
            terrain: (elevs_f64[i] * 10.0).round() / 10.0,
            terrain_bulge: (fresnel.terrain_bulge[i] * 100.0).round() / 100.0,
            los: (fresnel.los_h[i] * 100.0).round() / 100.0,
            fresnel_upper: ((fresnel.los_h[i] + fresnel.fresnel_r[i]) * 100.0).round() / 100.0,
            fresnel_lower: ((fresnel.los_h[i] - fresnel.fresnel_r[i]) * 100.0).round() / 100.0,
            fresnel_60: ((fresnel.los_h[i] - 0.6 * fresnel.fresnel_r[i]) * 100.0).round() / 100.0,
            blocked: fresnel.obstructs_los[i],
            violates_f1: fresnel.violates_f1[i],
            violates_f60: fresnel.violates_f60[i],
        })
        .collect();

    let max_pts = 400;
    if profile_data.len() > max_pts {
        let step = (profile_data.len() - 1) as f64 / (max_pts - 1) as f64;
        profile_data = (0..max_pts)
            .map(|i| {
                let idx = ((i as f64 * step) as usize).min(profile_data.len() - 1);
                profile_data[idx].clone()
            })
            .collect();
    }

    let eirp_dbm = req.tx_power_dbm + req.tx_gain_dbi - req.cable_loss_db;
    let prx_dbm = eirp_dbm + req.rx_gain_dbi - result.loss_db;
    let margin_db = prx_dbm - req.rx_sensitivity_dbm;
    let fspl_db = if dist_m > 0.0 && req.freq_mhz > 0.0 {
        20.0 * (dist_m / 1000.0).log10() + 20.0 * req.freq_mhz.log10() + 32.44
    } else {
        0.0
    };

    let mut horizons = Vec::new();
    if result.d_hzn_tx_m > 0.0 && result.d_hzn_tx_m < dist_m {
        horizons.push(Horizon {
            role: "tx_horizon".into(),
            d_m: (result.d_hzn_tx_m * 10.0).round() / 10.0,
        });
    }
    if result.d_hzn_rx_m > 0.0 && result.d_hzn_rx_m < dist_m {
        horizons.push(Horizon {
            role: "rx_horizon".into(),
            d_m: ((dist_m - result.d_hzn_rx_m) * 10.0).round() / 10.0,
        });
    }

    Json(P2PResponse {
        distance_m: (dist_m * 10.0).round() / 10.0,
        profile: profile_data,
        loss_db: (result.loss_db * 100.0).round() / 100.0,
        mode: result.mode,
        mode_name: PROP_MODE_NAMES
            .get(result.mode as usize)
            .unwrap_or(&"Unknown")
            .to_string(),
        warnings: result.warnings,
        link_budget: LinkBudget {
            tx_power_dbm: (req.tx_power_dbm * 100.0).round() / 100.0,
            tx_gain_dbi: (req.tx_gain_dbi * 100.0).round() / 100.0,
            rx_gain_dbi: (req.rx_gain_dbi * 100.0).round() / 100.0,
            cable_loss_db: (req.cable_loss_db * 100.0).round() / 100.0,
            eirp_dbm: (eirp_dbm * 100.0).round() / 100.0,
            fspl_db: (fspl_db * 100.0).round() / 100.0,
            itm_loss_db: (result.loss_db * 100.0).round() / 100.0,
            excess_loss_db: ((result.loss_db - fspl_db) * 100.0).round() / 100.0,
            prx_dbm: (prx_dbm * 100.0).round() / 100.0,
            rx_sensitivity_dbm: (req.rx_sensitivity_dbm * 100.0).round() / 100.0,
            margin_db: (margin_db * 100.0).round() / 100.0,
        },
        horizons,
        flags: Flags {
            los_blocked: any_blockage,
            fresnel_f1_violated: any_f1,
            fresnel_60_violated: any_f60,
        },
        k_factor: (req.k_factor * 1000.0).round() / 1000.0,
        intermediates: Intermediates {
            d_hzn_tx_m: (result.d_hzn_tx_m * 10.0).round() / 10.0,
            d_hzn_rx_m: (result.d_hzn_rx_m * 10.0).round() / 10.0,
            h_e_tx_m: (result.h_e_tx_m * 100.0).round() / 100.0,
            h_e_rx_m: (result.h_e_rx_m * 100.0).round() / 100.0,
            delta_h_m: (result.delta_h_m * 100.0).round() / 100.0,
            a_ref_db: (result.a_ref_db * 100.0).round() / 100.0,
        },
    })
}
