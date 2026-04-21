use crate::elevation::ElevationGrid;
use crate::error::AppError;
use crate::fresnel::fresnel_profile_analysis;
use crate::itm_bridge::{itm_p2p_loss, ITM_LOSS_SENTINEL, PROP_MODE_NAMES};
use crate::models::*;
use crate::rounding::{round1, round2, round3};
use crate::signal_levels::sample_line_from_grid;
use crate::terrain::{build_pfl, haversine_m};
use axum::Json;

/// P2P elevation sampling step in meters
const P2P_STEP_M: f64 = 30.0;
/// Maximum number of profile points for P2P analysis
const P2P_MAX_PROFILE_PTS: usize = 1000;
/// Maximum number of profile data points returned in the response
const P2P_MAX_RESPONSE_PTS: usize = 400;
/// ITM mdvar parameter (point-to-point mode)
const P2P_MDVAR: i32 = 12;

const C: f64 = 299_792_458.0;

pub async fn p2p_handler(Json(req): Json<P2PRequest>) -> Result<Json<P2PResponse>, AppError> {
    req.validate().map_err(AppError::unprocessable)?;

    let dist_m = haversine_m(req.tx.lat, req.tx.lon, req.rx.lat, req.rx.lon);
    let step_m = P2P_STEP_M;
    let n_pts = ((dist_m / step_m).round() as usize).clamp(2, P2P_MAX_PROFILE_PTS);

    let freq_mhz = req.freq_mhz;
    let tx_lat = req.tx.lat;
    let tx_lon = req.tx.lon;
    let rx_lat = req.rx.lat;
    let rx_lon = req.rx.lon;
    let tx_h_m = req.tx.h_m;
    let rx_h_m = req.rx.h_m;
    let climate = req.climate;
    let n0 = req.n0;
    let polarization = req.polarization;
    let epsilon = req.epsilon;
    let sigma = req.sigma;
    let time_pct = req.time_pct;
    let location_pct = req.location_pct;
    let situation_pct = req.situation_pct;
    let k_factor = req.k_factor;
    let tx_power_dbm = req.tx_power_dbm;
    let tx_gain_dbi = req.tx_gain_dbi;
    let rx_gain_dbi = req.rx_gain_dbi;
    let cable_loss_db = req.cable_loss_db;
    let rx_sensitivity_dbm = req.rx_sensitivity_dbm;

    let result = tokio::task::spawn_blocking(move || {
        let elev = ElevationGrid::fetch(
            tx_lat.min(rx_lat) - 0.01,
            tx_lon.min(rx_lon) - 0.01,
            tx_lat.max(rx_lat) + 0.01,
            tx_lon.max(rx_lon) + 0.01,
            n_pts,
            "glo30",
        );

        let sample_n = n_pts.max(2);
        let elevs = sample_line_from_grid(
            &elev.data,
            elev.min_lat,
            elev.max_lat,
            elev.min_lon,
            elev.max_lon,
            elev.n_lat,
            elev.n_lon,
            tx_lat,
            tx_lon,
            rx_lat,
            rx_lon,
            sample_n,
        );

        let elevs_f64: Vec<f64> = elevs
            .iter()
            .map(|&v| if v.is_nan() { 0.0 } else { v as f64 })
            .collect();
        let actual_step_m = dist_m / (sample_n - 1) as f64;

        let pfl = build_pfl(&elevs_f64, actual_step_m);

        let mdvar = P2P_MDVAR;
        let result = itm_p2p_loss(
            tx_h_m,
            rx_h_m,
            &pfl,
            climate,
            n0,
            freq_mhz,
            polarization,
            epsilon,
            sigma,
            mdvar,
            time_pct,
            location_pct,
            situation_pct,
        );

        let tx_elev = elevs_f64[0];
        let rx_elev = elevs_f64[elevs_f64.len() - 1];
        let tx_antenna_h = tx_elev + tx_h_m;
        let rx_antenna_h = rx_elev + rx_h_m;

        let distances: Vec<f64> = (0..sample_n).map(|i| i as f64 * actual_step_m).collect();
        let wavelength_m = C / (freq_mhz * 1e6);

        let fresnel = fresnel_profile_analysis(
            &distances,
            &elevs_f64,
            tx_antenna_h,
            rx_antenna_h,
            dist_m,
            wavelength_m,
            k_factor,
        );

        let any_blockage = fresnel.obstructs_los.iter().any(|&b| b);
        let any_f1 = fresnel.violates_f1.iter().any(|&b| b);
        let any_f60 = fresnel.violates_f60.iter().any(|&b| b);

        let mut profile_data: Vec<ProfilePoint> = distances
            .iter()
            .enumerate()
            .map(|(i, &d)| ProfilePoint {
                d: round1(d),
                terrain: round1(elevs_f64[i]),
                terrain_bulge: round2(fresnel.terrain_bulge[i]),
                los: round2(fresnel.los_h[i]),
                fresnel_upper: round2(fresnel.los_h[i] + fresnel.fresnel_r[i]),
                fresnel_lower: round2(fresnel.los_h[i] - fresnel.fresnel_r[i]),
                fresnel_60: round2(fresnel.los_h[i] - 0.6 * fresnel.fresnel_r[i]),
                blocked: fresnel.obstructs_los[i],
                violates_f1: fresnel.violates_f1[i],
                violates_f60: fresnel.violates_f60[i],
            })
            .collect();

        let profile_original_count = profile_data.len();

        let max_pts = P2P_MAX_RESPONSE_PTS;
        if profile_data.len() > max_pts {
            let step = (profile_data.len() - 1) as f64 / (max_pts - 1) as f64;
            profile_data = (0..max_pts)
                .map(|i| {
                    let idx = ((i as f64 * step) as usize).min(profile_data.len() - 1);
                    profile_data[idx]
                })
                .collect();
        }

        let eirp_dbm = tx_power_dbm + tx_gain_dbi - cable_loss_db;
        let prx_dbm = eirp_dbm + rx_gain_dbi - result.loss_db;
        let margin_db = prx_dbm - rx_sensitivity_dbm;
        let fspl_db = if dist_m > 0.0 && freq_mhz > 0.0 {
            20.0 * (dist_m / 1000.0).log10() + 20.0 * freq_mhz.log10() + 32.44
        } else {
            0.0
        };

        let mut horizons = Vec::new();
        if result.d_hzn_tx_m > 0.0 && result.d_hzn_tx_m < dist_m {
            horizons.push(Horizon {
                role: "tx_horizon".into(),
                d_m: round1(result.d_hzn_tx_m),
            });
        }
        if result.d_hzn_rx_m > 0.0 && result.d_hzn_rx_m < dist_m {
            horizons.push(Horizon {
                role: "rx_horizon".into(),
                d_m: round1(dist_m - result.d_hzn_rx_m),
            });
        }

        (
            result,
            profile_data,
            profile_original_count,
            horizons,
            eirp_dbm,
            prx_dbm,
            margin_db,
            fspl_db,
            any_blockage,
            any_f1,
            any_f60,
        )
    })
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    let (
        itm_result,
        profile_data,
        profile_original_count,
        horizons,
        eirp_dbm,
        prx_dbm,
        margin_db,
        fspl_db,
        any_blockage,
        any_f1,
        any_f60,
    ) = result;

    if itm_result.loss_db >= ITM_LOSS_SENTINEL {
        tracing::error!(
            "ITM returned sentinel loss for P2P: tx=({},{}) rx=({},{}) freq={}",
            req.tx.lat,
            req.tx.lon,
            req.rx.lat,
            req.rx.lon,
            req.freq_mhz
        );
        return Err(AppError::internal(
            "ITM propagation computation failed for the given parameters",
        ));
    }

    tracing::info!(
        "P2P analysis: dist={:.1}m loss={:.1}dB mode={} tx=({:.4},{:.4}) rx=({:.4},{:.4})",
        dist_m,
        itm_result.loss_db,
        itm_result.mode,
        req.tx.lat,
        req.tx.lon,
        req.rx.lat,
        req.rx.lon
    );

    Ok(Json(P2PResponse {
        distance_m: round1(dist_m),
        profile: profile_data,
        profile_original_count,
        loss_db: round2(itm_result.loss_db),
        mode: itm_result.mode,
        mode_name: PROP_MODE_NAMES
            .get(itm_result.mode as usize)
            .unwrap_or(&"Unknown")
            .to_string(),
        warnings: itm_result.warnings,
        link_budget: LinkBudget {
            tx_power_dbm: round2(req.tx_power_dbm),
            tx_gain_dbi: round2(req.tx_gain_dbi),
            rx_gain_dbi: round2(req.rx_gain_dbi),
            cable_loss_db: round2(req.cable_loss_db),
            eirp_dbm: round2(eirp_dbm),
            fspl_db: round2(fspl_db),
            itm_loss_db: round2(itm_result.loss_db),
            excess_loss_db: round2(itm_result.loss_db - fspl_db),
            prx_dbm: round2(prx_dbm),
            rx_sensitivity_dbm: round2(req.rx_sensitivity_dbm),
            margin_db: round2(margin_db),
        },
        horizons,
        flags: Flags {
            los_blocked: any_blockage,
            fresnel_f1_violated: any_f1,
            fresnel_60_violated: any_f60,
        },
        k_factor: round3(req.k_factor),
        intermediates: Intermediates {
            d_hzn_tx_m: round1(itm_result.d_hzn_tx_m),
            d_hzn_rx_m: round1(itm_result.d_hzn_rx_m),
            h_e_tx_m: round2(itm_result.h_e_tx_m),
            h_e_rx_m: round2(itm_result.h_e_rx_m),
            delta_h_m: round2(itm_result.delta_h_m),
            a_ref_db: round2(itm_result.a_ref_db),
        },
    }))
}
