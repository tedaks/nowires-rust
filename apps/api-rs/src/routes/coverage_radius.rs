use crate::coverage::radius::{self, RadiusWorkerArgs};
use crate::coverage::workers::GridMeta;
use crate::elevation::ElevationGrid;
use crate::error::AppError;
use crate::itm_bridge::ITMParams;
use crate::models::*;
use axum::Json;

pub async fn coverage_radius_handler(
    Json(req): Json<CoverageRequest>,
) -> Result<Json<CoverageRadiusResponse>, AppError> {
    req.validate().map_err(AppError::unprocessable)?;

    let radius_km = req.radius_km.unwrap_or(100.0);
    let search_max_m = radius_km * 1000.0;
    let sweep_step_m = (500.0_f64).max(req.terrain_spacing_m * 2.0);
    let deg_per_m = 1.0 / 111_320.0;
    let pad_deg = 2.0 * req.terrain_spacing_m * deg_per_m;
    let padded_bbox_m = 2.0 * search_max_m + 4.0 * req.terrain_spacing_m;
    let elev_grid_n = req
        .elev_grid_n
        .unwrap_or_else(|| ((padded_bbox_m / req.terrain_spacing_m) as usize + 1).clamp(64, 1024));

    let lat_per_m = 1.0 / 111_320.0;
    let lon_per_m = 1.0 / (111_320.0 * req.tx.lat.to_radians().cos().max(0.01));
    let half_lat = search_max_m * lat_per_m;
    let half_lon = search_max_m * lon_per_m;

    let tx_lat = req.tx.lat;
    let tx_lon = req.tx.lon;
    let tx_h_m = req.tx.h_m;
    let rx_h_m = req.rx_h_m;
    let freq_mhz = req.freq_mhz;
    let polarization = req.polarization;
    let climate = req.climate;
    let n0 = req.n0;
    let epsilon = req.epsilon;
    let sigma = req.sigma;
    let time_pct = req.time_pct;
    let location_pct = req.location_pct;
    let situation_pct = req.situation_pct;
    let tx_power_dbm = req.tx_power_dbm;
    let tx_gain_dbi = req.tx_gain_dbi;
    let cable_loss_db = req.cable_loss_db;
    let rx_gain_dbi = req.rx_gain_dbi;
    let rx_sensitivity_dbm = req.rx_sensitivity_dbm;
    let antenna_az_deg = req.antenna_az_deg;
    let antenna_beamwidth_deg = req.antenna_beamwidth_deg;
    let elevation_source = req.elevation_source.clone();

    let result = tokio::task::spawn_blocking(move || {
        let elev = ElevationGrid::fetch(
            tx_lat - half_lat - pad_deg,
            tx_lon - half_lon - pad_deg,
            tx_lat + half_lat + pad_deg,
            tx_lon + half_lon + pad_deg,
            elev_grid_n,
            &elevation_source,
        );

        let eirp_dbm = tx_power_dbm + tx_gain_dbi - cable_loss_db;
        let grid_meta = GridMeta {
            min_lat: elev.min_lat,
            max_lat: elev.max_lat,
            min_lon: elev.min_lon,
            max_lon: elev.max_lon,
            n_lat: elev.n_lat,
            n_lon: elev.n_lon,
        };

        let args = RadiusWorkerArgs {
            tx_lat,
            tx_lon,
            params: ITMParams {
                tx_h_m,
                rx_h_m,
                climate,
                n0,
                f_mhz: freq_mhz,
                polarization,
                epsilon,
                sigma,
                time_pct,
                location_pct,
                situation_pct,
            },
            eirp_dbm,
            rx_gain_dbi,
            rx_sensitivity_dbm,
            antenna_az_deg,
            antenna_beamwidth_deg,
            sweep_step_m,
            search_max_m,
        };

        radius::compute_coverage_radius(&elev.data, &grid_meta, &args)
    })
    .await
    .map_err(|e| AppError::internal(e.to_string()))?;

    tracing::info!(
        "Coverage radius: tx=({:.4},{:.4}) max={:.1}km avg={:.1}km min={:.1}km",
        req.tx.lat,
        req.tx.lon,
        result.max_radius_km,
        result.avg_radius_km,
        result.min_radius_km,
    );

    Ok(Json(result))
}
