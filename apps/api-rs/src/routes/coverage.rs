use crate::antenna::antenna_gain_factor;
use crate::coverage::render::{render_coverage_result, RenderParams};
use crate::coverage::workers::{self, GridMeta, ITMWorkerArgs};
use crate::elevation::ElevationGrid;
use crate::models::*;
use axum::Json;
use rayon::prelude::*;

pub async fn coverage_handler(Json(req): Json<CoverageRequest>) -> Json<CoverageResponse> {
    let radius_km = req.radius_km.unwrap_or(50.0);
    let radius_m = radius_km * 1000.0;
    let deg_per_m = 1.0 / 111_320.0;
    let pad_deg = 2.0 * req.terrain_spacing_m * deg_per_m;
    let padded_bbox_m = 2.0 * radius_m + 4.0 * req.terrain_spacing_m;
    let elev_grid_n = req.elev_grid_n.unwrap_or_else(|| {
        ((padded_bbox_m / req.terrain_spacing_m) as usize + 1).clamp(64, req.grid_size + 64)
    });

    let lat_per_m = 1.0 / 111_320.0;
    let lon_per_m = 1.0 / (111_320.0 * req.tx.lat.to_radians().cos().max(0.01));
    let half_lat = radius_m * lat_per_m;
    let half_lon = radius_m * lon_per_m;
    let min_lat = req.tx.lat - half_lat;
    let max_lat = req.tx.lat + half_lat;
    let min_lon = req.tx.lon - half_lon;
    let max_lon = req.tx.lon + half_lon;

    let elev = ElevationGrid::fetch(
        min_lat - pad_deg,
        min_lon - pad_deg,
        max_lat + pad_deg,
        max_lon + pad_deg,
        elev_grid_n,
        &req.elevation_source,
    );

    let eirp_dbm = req.tx_power_dbm + req.tx_gain_dbi - req.cable_loss_db;

    let lats: Vec<f64> = (0..req.grid_size)
        .map(|i| min_lat + (max_lat - min_lat) * i as f64 / (req.grid_size - 1) as f64)
        .collect();
    let lons: Vec<f64> = (0..req.grid_size)
        .map(|j| min_lon + (max_lon - min_lon) * j as f64 / (req.grid_size - 1) as f64)
        .collect();

    let mut prx_grid = vec![vec![f32::NAN; req.grid_size]; req.grid_size];
    let mut loss_grid = vec![vec![f32::NAN; req.grid_size]; req.grid_size];

    let grid_meta = GridMeta {
        min_lat: elev.min_lat,
        max_lat: elev.max_lat,
        min_lon: elev.min_lon,
        max_lon: elev.max_lon,
        n_lat: elev.n_lat,
        n_lon: elev.n_lon,
        tx_lat: req.tx.lat,
        tx_lon: req.tx.lon,
    };

    let max_profile_pts = 75usize;

    let mut pixels_attempted = 0usize;
    let mut pixels_failed = 0usize;

    let mut args_vec: Vec<ITMWorkerArgs> = Vec::new();
    let mut positions: Vec<(usize, usize)> = Vec::new();

    for (i, &lat) in lats.iter().enumerate().take(req.grid_size) {
        for (j, &lon) in lons.iter().enumerate().take(req.grid_size) {
            let dlat_m = (lat - req.tx.lat) / lat_per_m;
            let dlon_m = (lon - req.tx.lon) / lon_per_m;
            let d_m = (dlat_m * dlat_m + dlon_m * dlon_m).sqrt();
            if d_m < 50.0 || d_m > radius_m {
                continue;
            }
            let bearing = (dlon_m.atan2(dlat_m).to_degrees() + 360.0) % 360.0;
            let n_pts = ((d_m / req.profile_step_m).round() as usize).clamp(3, max_profile_pts);
            let step_m = d_m / (n_pts - 1) as f64;
            let ant_gain_adj =
                antenna_gain_factor(bearing, req.antenna_az_deg, req.antenna_beamwidth_deg, 25.0);

            let args = ITMWorkerArgs {
                i,
                j,
                target_lat: lat,
                target_lon: lon,
                step_m,
                n_pts,
                tx_h_m: req.tx.h_m,
                rx_h_m: req.rx_h_m,
                climate: req.climate,
                n0: req.n0,
                f_mhz: req.freq_mhz,
                polarization: req.polarization,
                epsilon: req.epsilon,
                sigma: req.sigma,
                time_pct: req.time_pct,
                location_pct: req.location_pct,
                situation_pct: req.situation_pct,
                eirp_dbm,
                ant_gain_adj,
                rx_gain_dbi: req.rx_gain_dbi,
            };
            args_vec.push(args);
            positions.push((i, j));
            pixels_attempted += 1;
        }
    }

    let grid_data = &elev.data;
    let results: Vec<Option<workers::ITMWorkerResult>> = args_vec
        .into_par_iter()
        .map(|args| workers::itm_worker(grid_data, &grid_meta, &args))
        .collect();

    for ((_i, _j), opt_res) in positions.into_iter().zip(results) {
        match opt_res {
            Some(res) => {
                loss_grid[res.i][res.j] = res.loss_db as f32;
                prx_grid[res.i][res.j] = res.prx as f32;
            }
            None => {
                pixels_failed += 1;
            }
        }
    }

    Json(render_coverage_result(
        &prx_grid,
        &loss_grid,
        RenderParams {
            grid_size: req.grid_size,
            elev_min_lat: elev.min_lat,
            elev_min_lon: elev.min_lon,
            elev_max_lat: elev.max_lat,
            elev_max_lon: elev.max_lon,
            elev_n_lat: elev.n_lat,
            elev_n_lon: elev.n_lon,
            tx_lat: req.tx.lat,
            eirp_dbm,
            rx_sensitivity_dbm: req.rx_sensitivity_dbm,
            deg_per_m,
            min_lat,
            max_lat,
            min_lon,
            max_lon,
            pixels_attempted,
            pixels_failed,
        },
    ))
}
