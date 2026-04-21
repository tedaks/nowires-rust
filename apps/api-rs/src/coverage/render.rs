use crate::fresnel::apply_coverage_colors;
use crate::models::*;
use crate::rounding::{round1, round2};
use crate::signal_levels::{COLORS, SIGNAL_LEVELS, THRESHOLDS};
use base64::{engine::general_purpose::STANDARD, Engine};

pub(crate) struct RenderParams {
    pub grid_size: usize,
    pub elev_min_lat: f64,
    pub elev_min_lon: f64,
    pub elev_max_lat: f64,
    pub elev_max_lon: f64,
    pub elev_n_lat: usize,
    pub elev_n_lon: usize,
    pub tx_lat: f64,
    pub eirp_dbm: f64,
    pub rx_sensitivity_dbm: f64,
    pub deg_per_m: f64,
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
    pub pixels_attempted: usize,
    pub pixels_failed: usize,
    pub profile_step_m: f64,
}

pub fn render_coverage_result(
    prx_grid: &[Vec<f32>],
    loss_grid: &[Vec<f32>],
    elev_data: &[Vec<f32>],
    params: RenderParams,
) -> CoverageResponse {
    let rgba = apply_coverage_colors(
        prx_grid,
        &THRESHOLDS,
        &COLORS,
        params.grid_size,
        params.grid_size,
    );

    let mut png_data = Vec::new();
    {
        use image::ImageEncoder;
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        let flat_rgba: Vec<u8> = rgba
            .iter()
            .flat_map(|row| row.iter().flat_map(|pixel| pixel.iter().copied()))
            .collect();
        encoder
            .write_image(
                &flat_rgba,
                params.grid_size as u32,
                params.grid_size as u32,
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
    }
    let png_b64 = STANDARD.encode(&png_data);

    let mut pixels_valid = 0usize;
    let mut prx_min = f64::MAX;
    let mut prx_max = f64::MIN;
    let mut loss_min = f64::MAX;
    let mut loss_max = f64::MIN;
    let mut above_sens = 0usize;

    for row in prx_grid {
        for &v in row {
            if !v.is_nan() {
                pixels_valid += 1;
                let v64 = v as f64;
                if v64 < prx_min {
                    prx_min = v64;
                }
                if v64 > prx_max {
                    prx_max = v64;
                }
                if v64 >= params.rx_sensitivity_dbm {
                    above_sens += 1;
                }
            }
        }
    }
    for row in loss_grid {
        for &v in row {
            if !v.is_nan() {
                let v64 = v as f64;
                if v64 < loss_min {
                    loss_min = v64;
                }
                if v64 > loss_max {
                    loss_max = v64;
                }
            }
        }
    }

    let pct_above = if pixels_valid > 0 {
        above_sens as f64 / pixels_valid as f64 * 100.0
    } else {
        0.0
    };
    let terr_spacing =
        ((params.elev_max_lat - params.elev_min_lat) / params.elev_n_lat as f64 / 2.0
            + (params.elev_max_lon - params.elev_min_lon) / params.elev_n_lon as f64 / 2.0
                * params.tx_lat.to_radians().cos())
            / params.deg_per_m;

    let legend: Vec<LegendEntry> = SIGNAL_LEVELS
        .iter()
        .map(|sl| LegendEntry {
            threshold_dbm: sl.threshold_dbm,
            rgba: sl.rgba,
            label: sl.label.to_string(),
        })
        .collect();

    let mut elev_min = f64::MAX;
    let mut elev_max = f64::MIN;
    let mut elev_sum = 0.0f64;
    let mut elev_count = 0usize;
    for row in elev_data {
        for &v in row {
            if v.is_finite() {
                let v64 = v as f64;
                if v64 < elev_min {
                    elev_min = v64;
                }
                if v64 > elev_max {
                    elev_max = v64;
                }
                elev_sum += v64;
                elev_count += 1;
            }
        }
    }
    let elev_mean = if elev_count > 0 {
        elev_sum / elev_count as f64
    } else {
        0.0
    };
    let mut variance_sum = 0.0f64;
    for row in elev_data {
        for &v in row {
            if v.is_finite() {
                let diff = v as f64 - elev_mean;
                variance_sum += diff * diff;
            }
        }
    }
    let elev_std = if elev_count > 1 {
        (variance_sum / (elev_count - 1) as f64).sqrt()
    } else {
        0.0
    };

    CoverageResponse {
        png_base64: png_b64,
        bounds: [
            [params.min_lat, params.min_lon],
            [params.max_lat, params.max_lon],
        ],
        legend,
        eirp_dbm: round2(params.eirp_dbm),
        rx_sensitivity_dbm: params.rx_sensitivity_dbm,
        stats: CoverageStats {
            pixels_total: params.grid_size * params.grid_size,
            pixels_valid,
            pixels_attempted: params.pixels_attempted,
            pixels_failed: params.pixels_failed,
            prx_min_dbm: if pixels_valid > 0 {
                Some(round2(prx_min))
            } else {
                None
            },
            prx_max_dbm: if pixels_valid > 0 {
                Some(round2(prx_max))
            } else {
                None
            },
            pct_above_sensitivity: round1(pct_above),
            terrain_grid_n: params.elev_n_lat,
            terrain_spacing_m: round1(terr_spacing),
            terrain_elev_min_m: if elev_count > 0 {
                round2(elev_min)
            } else {
                0.0
            },
            terrain_elev_max_m: if elev_count > 0 {
                round2(elev_max)
            } else {
                0.0
            },
            terrain_elev_std_m: round2(elev_std),
            loss_min_db: if pixels_valid > 0 {
                Some(round2(loss_min))
            } else {
                None
            },
            loss_max_db: if pixels_valid > 0 {
                Some(round2(loss_max))
            } else {
                None
            },
        },
        from_cache: false,
        profile_step_m: round2(params.profile_step_m),
    }
}
