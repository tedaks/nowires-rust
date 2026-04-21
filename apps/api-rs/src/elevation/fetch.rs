use std::path::PathBuf;

fn glo30_tile_path(lat: i32, lon: i32) -> PathBuf {
    let lat_str = if lat >= 0 {
        format!("N{:02}", lat)
    } else {
        format!("S{:02}", lat.abs())
    };
    let lon_str = if lon >= 0 {
        format!("E{:03}", lon)
    } else {
        format!("W{:03}", lon.abs())
    };
    let root = std::env::var("GLO30_TILES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("../../data/glo30");
            d
        });
    root.join(format!(
        "Copernicus_DSM_COG_10_{}_00_{}_00_DEM.tif",
        lat_str, lon_str
    ))
}

fn srtm1_tile_path(lat: i32, lon: i32) -> PathBuf {
    let lat_str = if lat >= 0 {
        format!("N{:02}", lat)
    } else {
        format!("S{:02}", lat.abs())
    };
    let lon_str = if lon >= 0 {
        format!("E{:03}", lon)
    } else {
        format!("W{:03}", lon.abs())
    };
    let root = std::env::var("SRTM1_TILES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("../../data/srtm1");
            d
        });
    root.join(&lat_str)
        .join(format!("{}{}.tif", lat_str, lon_str))
}

#[allow(clippy::too_many_arguments)]
fn read_geotiff_grid(
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
    lats: &[f64],
    lons: &[f64],
    n: usize,
    tile_path_fn: &dyn Fn(i32, i32) -> PathBuf,
) -> Option<Vec<Vec<f32>>> {
    let mut result = vec![vec![f32::NAN; n]; n];

    for lat_t in (min_lat.floor() as i32)..=(max_lat.floor() as i32) {
        for lon_t in (min_lon.floor() as i32)..=(max_lon.floor() as i32) {
            let path = tile_path_fn(lat_t, lon_t);
            if !path.exists() {
                continue;
            }
            if let Ok(dataset) = gdal::Dataset::open(&path) {
                let raster = dataset.rasterband(1).ok()?;
                let transform = dataset.geo_transform().ok()?;
                let (ds_rows, ds_cols) = (raster.y_size(), raster.x_size());
                let nodata = raster.no_data_value().map(|v| v as f32);

                // Bulk read entire rasterband once
                let buf = match raster.read_as::<f32>(
                    (0, 0),
                    (ds_cols, ds_rows),
                    (ds_cols, ds_rows),
                    None,
                ) {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                for (i, &lat) in lats.iter().enumerate() {
                    for (j, &lon) in lons.iter().enumerate() {
                        if lat < lat_t as f64 - 1e-6 || lat >= (lat_t + 1) as f64 + 1e-6 {
                            continue;
                        }
                        if lon < lon_t as f64 - 1e-6 || lon >= (lon_t + 1) as f64 + 1e-6 {
                            continue;
                        }
                        let col = ((lon - transform[0]) / transform[1]) as isize;
                        let row = ((lat - transform[3]) / transform[5]) as isize;
                        if col < 0 || row < 0 || col >= ds_cols as isize || row >= ds_rows as isize
                        {
                            continue;
                        }
                        let v = buf.data()[(row as usize) * ds_cols + col as usize];
                        if let Some(nd) = nodata {
                            if (v - nd).abs() < 1e-3 {
                                continue;
                            }
                        }
                        result[i][j] = v;
                    }
                }
            }
        }
    }

    if result.iter().all(|row| row.iter().all(|v| v.is_nan())) {
        return None;
    }
    Some(result)
}

fn interpolate_grid_nans(grid: &mut [Vec<f32>]) {
    for row in grid.iter_mut() {
        for i in 0..row.len() {
            if row[i].is_nan() {
                let left = row[..i].iter().rev().find(|v| !v.is_nan()).copied();
                let right = row[i + 1..].iter().find(|v| !v.is_nan()).copied();
                row[i] = match (left, right) {
                    (Some(l), Some(r)) => (l + r) / 2.0,
                    (Some(l), None) => l,
                    (None, Some(r)) => r,
                    _ => f32::NAN,
                };
            }
        }
    }
}

pub fn fetch_grid(
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
    lats: &[f64],
    lons: &[f64],
    source: &str,
) -> Vec<Vec<f32>> {
    let n = lats.len();

    if source == "glo30" {
        if let Some(mut data) = read_geotiff_grid(
            min_lat,
            min_lon,
            max_lat,
            max_lon,
            lats,
            lons,
            n,
            &glo30_tile_path,
        ) {
            interpolate_grid_nans(&mut data);
            return data;
        }
        if let Some(mut data) = read_geotiff_grid(
            min_lat,
            min_lon,
            max_lat,
            max_lon,
            lats,
            lons,
            n,
            &srtm1_tile_path,
        ) {
            interpolate_grid_nans(&mut data);
            return data;
        }
    } else if source == "srtm1" {
        if let Some(mut data) = read_geotiff_grid(
            min_lat,
            min_lon,
            max_lat,
            max_lon,
            lats,
            lons,
            n,
            &srtm1_tile_path,
        ) {
            interpolate_grid_nans(&mut data);
            return data;
        }
    }

    vec![vec![0.0f32; n]; n]
}
