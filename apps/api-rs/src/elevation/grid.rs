use std::path::PathBuf;

pub struct ElevationGrid {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
    pub data: Vec<Vec<f32>>,
    pub n_lat: usize,
    pub n_lon: usize,
    #[allow(dead_code)]
    pub d_lat: f64,
    #[allow(dead_code)]
    pub d_lon: f64,
}

impl ElevationGrid {
    pub fn new(
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
        data: Vec<Vec<f32>>,
    ) -> Self {
        let n_lat = data.len();
        let n_lon = if n_lat > 0 { data[0].len() } else { 0 };
        let d_lat = if n_lat > 1 {
            (max_lat - min_lat) / (n_lat - 1) as f64
        } else {
            0.0
        };
        let d_lon = if n_lon > 1 {
            (max_lon - min_lon) / (n_lon - 1) as f64
        } else {
            0.0
        };
        Self {
            min_lat,
            min_lon,
            max_lat,
            max_lon,
            data,
            n_lat,
            n_lon,
            d_lat,
            d_lon,
        }
    }

    pub fn fetch(
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
        n: usize,
        source: &str,
    ) -> Self {
        let cached = Self::try_load_cache(min_lat, min_lon, max_lat, max_lon, n, source);
        if let Some(grid) = cached {
            return grid;
        }

        let lats: Vec<f64> = (0..n)
            .map(|i| min_lat + (max_lat - min_lat) * i as f64 / (n - 1) as f64)
            .collect();
        let lons: Vec<f64> = (0..n)
            .map(|i| min_lon + (max_lon - min_lon) * i as f64 / (n - 1) as f64)
            .collect();

        let data =
            super::fetch::fetch_grid(min_lat, min_lon, max_lat, max_lon, &lats, &lons, source);

        let grid = Self::new(min_lat, min_lon, max_lat, max_lon, data);
        grid.save_cache(n, source);
        grid
    }

    fn cache_key(
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
        n: usize,
        source: &str,
    ) -> PathBuf {
        let s =
            format!("elev_{source}_{min_lat:.4}_{min_lon:.4}_{max_lat:.4}_{max_lon:.4}_{n}.json");
        let safe: String = s
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let cache_dir = std::env::var("ELEV_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                d.push("../../data/elev_cache");
                d
            });
        cache_dir.join(format!("{}.json", safe))
    }

    fn try_load_cache(
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
        n: usize,
        source: &str,
    ) -> Option<Self> {
        let path = Self::cache_key(min_lat, min_lon, max_lat, max_lon, n, source);
        if !path.exists() {
            return None;
        }
        match std::fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<ElevationGridCache>(&bytes) {
                Ok(cached) => Some(Self::new(
                    cached.min_lat,
                    cached.min_lon,
                    cached.max_lat,
                    cached.max_lon,
                    cached.data,
                )),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    fn save_cache(&self, n: usize, source: &str) {
        let path = Self::cache_key(
            self.min_lat,
            self.min_lon,
            self.max_lat,
            self.max_lon,
            n,
            source,
        );
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let cached = ElevationGridCache {
            min_lat: self.min_lat,
            min_lon: self.min_lon,
            max_lat: self.max_lat,
            max_lon: self.max_lon,
            data: self.data.clone(),
        };
        if let Ok(json) = serde_json::to_string(&cached) {
            let tmp_path = path.with_extension("json.tmp");
            if std::fs::write(&tmp_path, json.as_bytes()).is_ok() {
                let _ = std::fs::rename(&tmp_path, &path);
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ElevationGridCache {
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
    data: Vec<Vec<f32>>,
}
