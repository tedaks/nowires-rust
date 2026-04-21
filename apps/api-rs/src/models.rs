pub mod response;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
    #[serde(default = "default_h_m")]
    pub h_m: f64,
}

impl Coordinates {
    pub fn validate(&self, label: &str) -> Result<(), String> {
        if !self.lat.is_finite() || self.lat < -90.0 || self.lat > 90.0 {
            return Err(format!(
                "{} latitude must be in [-90, 90], got {}",
                label, self.lat
            ));
        }
        if !self.lon.is_finite() || self.lon < -180.0 || self.lon > 180.0 {
            return Err(format!(
                "{} longitude must be in [-180, 180], got {}",
                label, self.lon
            ));
        }
        if !self.h_m.is_finite() || self.h_m < 0.0 || self.h_m > 10000.0 {
            return Err(format!(
                "{} height must be in [0, 10000], got {}",
                label, self.h_m
            ));
        }
        Ok(())
    }
}

fn default_h_m() -> f64 {
    30.0
}

#[derive(Deserialize)]
pub struct P2PRequest {
    pub tx: Coordinates,
    pub rx: Coordinates,
    #[serde(default = "default_freq")]
    pub freq_mhz: f64,
    #[serde(default)]
    pub polarization: i32,
    #[serde(default = "default_climate")]
    pub climate: i32,
    #[serde(default = "default_n0")]
    pub n0: f64,
    #[serde(default = "default_epsilon")]
    pub epsilon: f64,
    #[serde(default = "default_sigma")]
    pub sigma: f64,
    #[serde(default = "default_time_pct")]
    pub time_pct: f64,
    #[serde(default = "default_location_pct")]
    pub location_pct: f64,
    #[serde(default = "default_situation_pct")]
    pub situation_pct: f64,
    #[serde(default = "default_k_factor")]
    pub k_factor: f64,
    #[serde(default = "default_tx_power")]
    pub tx_power_dbm: f64,
    #[serde(default = "default_tx_gain")]
    pub tx_gain_dbi: f64,
    #[serde(default = "default_rx_gain")]
    pub rx_gain_dbi: f64,
    #[serde(default = "default_cable_loss")]
    pub cable_loss_db: f64,
    #[serde(default = "default_rx_sensitivity")]
    pub rx_sensitivity_dbm: f64,
}

fn default_freq() -> f64 {
    300.0
}
fn default_climate() -> i32 {
    1
}
fn default_n0() -> f64 {
    301.0
}
fn default_epsilon() -> f64 {
    15.0
}
fn default_sigma() -> f64 {
    0.005
}
fn default_time_pct() -> f64 {
    50.0
}
fn default_location_pct() -> f64 {
    50.0
}
fn default_situation_pct() -> f64 {
    50.0
}
fn default_k_factor() -> f64 {
    4.0 / 3.0
}
fn default_tx_power() -> f64 {
    43.0
}
fn default_tx_gain() -> f64 {
    8.0
}
fn default_rx_gain() -> f64 {
    2.0
}
fn default_cable_loss() -> f64 {
    2.0
}
fn default_rx_sensitivity() -> f64 {
    -100.0
}

impl P2PRequest {
    pub fn validate(&self) -> Result<(), String> {
        self.tx.validate("TX")?;
        self.rx.validate("RX")?;
        if !self.freq_mhz.is_finite() || self.freq_mhz <= 0.0 {
            return Err(format!("freq_mhz must be > 0, got {}", self.freq_mhz));
        }
        if self.freq_mhz > 100_000.0 {
            return Err(format!("freq_mhz must be <= 100000, got {}", self.freq_mhz));
        }
        if self.k_factor <= 0.0 {
            return Err(format!("k_factor must be > 0, got {}", self.k_factor));
        }
        validate_percent("time_pct", self.time_pct)?;
        validate_percent("location_pct", self.location_pct)?;
        validate_percent("situation_pct", self.situation_pct)?;
        Ok(())
    }
}

fn validate_percent(name: &str, v: f64) -> Result<(), String> {
    if !(0.0..=100.0).contains(&v) {
        return Err(format!("{} must be in [0, 100], got {}", name, v));
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct CoverageRequest {
    pub tx: Coordinates,
    #[serde(default = "default_rx_h")]
    pub rx_h_m: f64,
    #[serde(default = "default_freq")]
    pub freq_mhz: f64,
    pub radius_km: Option<f64>,
    #[serde(default = "default_grid_size")]
    pub grid_size: usize,
    #[serde(default = "default_profile_step")]
    pub profile_step_m: f64,
    #[serde(default = "default_terrain_spacing")]
    pub terrain_spacing_m: f64,
    pub elev_grid_n: Option<usize>,
    #[serde(default = "default_elevation_source")]
    pub elevation_source: String,
    #[serde(default)]
    pub polarization: i32,
    #[serde(default = "default_climate")]
    pub climate: i32,
    #[serde(default = "default_n0")]
    pub n0: f64,
    #[serde(default = "default_epsilon")]
    pub epsilon: f64,
    #[serde(default = "default_sigma")]
    pub sigma: f64,
    #[serde(default = "default_time_pct")]
    pub time_pct: f64,
    #[serde(default = "default_location_pct")]
    pub location_pct: f64,
    #[serde(default = "default_situation_pct")]
    pub situation_pct: f64,
    #[serde(default = "default_tx_power")]
    pub tx_power_dbm: f64,
    #[serde(default = "default_tx_gain")]
    pub tx_gain_dbi: f64,
    #[serde(default = "default_rx_gain")]
    pub rx_gain_dbi: f64,
    #[serde(default = "default_cable_loss")]
    pub cable_loss_db: f64,
    #[serde(default = "default_rx_sensitivity")]
    pub rx_sensitivity_dbm: f64,
    pub antenna_az_deg: Option<f64>,
    #[serde(default = "default_beamwidth")]
    pub antenna_beamwidth_deg: f64,
}

fn default_rx_h() -> f64 {
    10.0
}
fn default_grid_size() -> usize {
    192
}
fn default_profile_step() -> f64 {
    250.0
}
fn default_terrain_spacing() -> f64 {
    300.0
}
fn default_elevation_source() -> String {
    "glo30".into()
}
fn default_beamwidth() -> f64 {
    360.0
}

impl CoverageRequest {
    pub fn validate(&self) -> Result<(), String> {
        self.tx.validate("TX")?;
        if !self.rx_h_m.is_finite() || self.rx_h_m < 0.0 || self.rx_h_m > 10000.0 {
            return Err(format!("rx_h_m must be in [0, 10000], got {}", self.rx_h_m));
        }
        if !self.freq_mhz.is_finite() || self.freq_mhz <= 0.0 {
            return Err(format!("freq_mhz must be > 0, got {}", self.freq_mhz));
        }
        if self.freq_mhz > 100_000.0 {
            return Err(format!("freq_mhz must be <= 100000, got {}", self.freq_mhz));
        }
        if let Some(r) = self.radius_km {
            if !r.is_finite() || r <= 0.0 {
                return Err(format!("radius_km must be > 0, got {}", r));
            }
        }
        if self.grid_size < 2 {
            return Err(format!("grid_size must be >= 2, got {}", self.grid_size));
        }
        if self.grid_size > 512 {
            return Err(format!("grid_size must be <= 512, got {}", self.grid_size));
        }
        if !self.terrain_spacing_m.is_finite() || self.terrain_spacing_m <= 0.0 {
            return Err(format!(
                "terrain_spacing_m must be > 0, got {}",
                self.terrain_spacing_m
            ));
        }
        validate_percent("time_pct", self.time_pct)?;
        validate_percent("location_pct", self.location_pct)?;
        validate_percent("situation_pct", self.situation_pct)?;
        Ok(())
    }
}

pub use response::*;
