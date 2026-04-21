use crate::coverage::radius::{self, RadiusGridMeta, RadiusWorkerArgs};
use crate::elevation::ElevationGrid;
use crate::models::*;
use axum::Json;

pub async fn coverage_radius_handler(
    Json(req): Json<CoverageRequest>,
) -> Json<CoverageRadiusResponse> {
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

    let elev = ElevationGrid::fetch(
        req.tx.lat - half_lat - pad_deg,
        req.tx.lon - half_lon - pad_deg,
        req.tx.lat + half_lat + pad_deg,
        req.tx.lon + half_lon + pad_deg,
        elev_grid_n,
        &req.elevation_source,
    );

    let eirp_dbm = req.tx_power_dbm + req.tx_gain_dbi - req.cable_loss_db;
    let grid_meta = RadiusGridMeta {
        min_lat: elev.min_lat,
        max_lat: elev.max_lat,
        min_lon: elev.min_lon,
        max_lon: elev.max_lon,
        n_lat: elev.n_lat,
        n_lon: elev.n_lon,
    };

    let args = RadiusWorkerArgs {
        tx_lat: req.tx.lat,
        tx_lon: req.tx.lon,
        tx_h_m: req.tx.h_m,
        rx_h_m: req.rx_h_m,
        f_mhz: req.freq_mhz,
        polarization: req.polarization,
        climate: req.climate,
        n0: req.n0,
        epsilon: req.epsilon,
        sigma: req.sigma,
        time_pct: req.time_pct,
        location_pct: req.location_pct,
        situation_pct: req.situation_pct,
        eirp_dbm,
        rx_gain_dbi: req.rx_gain_dbi,
        rx_sensitivity_dbm: req.rx_sensitivity_dbm,
        antenna_az_deg: req.antenna_az_deg,
        antenna_beamwidth_deg: req.antenna_beamwidth_deg,
        sweep_step_m,
        search_max_m,
    };

    Json(radius::compute_coverage_radius(
        &elev.data, &grid_meta, &args,
    ))
}
