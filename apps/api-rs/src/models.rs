use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
    #[serde(default = "default_h_m")]
    pub h_m: f64,
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

#[derive(Serialize)]
pub struct P2PResponse {
    pub distance_m: f64,
    pub profile: Vec<ProfilePoint>,
    pub loss_db: f64,
    pub mode: i32,
    pub mode_name: String,
    pub warnings: i32,
    pub link_budget: LinkBudget,
    pub horizons: Vec<Horizon>,
    pub flags: Flags,
    pub k_factor: f64,
    pub intermediates: Intermediates,
}

#[derive(Serialize, Clone)]
pub struct ProfilePoint {
    pub d: f64,
    pub terrain: f64,
    pub terrain_bulge: f64,
    pub los: f64,
    pub fresnel_upper: f64,
    pub fresnel_lower: f64,
    pub fresnel_60: f64,
    pub blocked: bool,
    pub violates_f1: bool,
    pub violates_f60: bool,
}

#[derive(Serialize)]
pub struct LinkBudget {
    pub tx_power_dbm: f64,
    pub tx_gain_dbi: f64,
    pub rx_gain_dbi: f64,
    pub cable_loss_db: f64,
    pub eirp_dbm: f64,
    pub fspl_db: f64,
    pub itm_loss_db: f64,
    pub excess_loss_db: f64,
    pub prx_dbm: f64,
    pub rx_sensitivity_dbm: f64,
    pub margin_db: f64,
}

#[derive(Serialize)]
pub struct Horizon {
    pub role: String,
    pub d_m: f64,
}

#[derive(Serialize)]
pub struct Flags {
    pub los_blocked: bool,
    pub fresnel_f1_violated: bool,
    pub fresnel_60_violated: bool,
}

#[derive(Serialize)]
pub struct Intermediates {
    pub d_hzn_tx_m: f64,
    pub d_hzn_rx_m: f64,
    pub h_e_tx_m: f64,
    pub h_e_rx_m: f64,
    pub delta_h_m: f64,
    pub a_ref_db: f64,
}

#[derive(Serialize)]
pub struct CoverageResponse {
    pub png_base64: String,
    pub bounds: [[f64; 2]; 2],
    pub legend: Vec<LegendEntry>,
    pub eirp_dbm: f64,
    pub rx_sensitivity_dbm: f64,
    pub stats: CoverageStats,
    pub from_cache: bool,
}

#[derive(Serialize)]
pub struct LegendEntry {
    pub threshold_dbm: f64,
    pub rgba: [u8; 4],
    pub label: String,
}

#[derive(Serialize)]
pub struct CoverageStats {
    pub pixels_total: usize,
    pub pixels_valid: usize,
    pub pixels_attempted: usize,
    pub pixels_failed: usize,
    pub prx_min_dbm: Option<f64>,
    pub prx_max_dbm: Option<f64>,
    pub pct_above_sensitivity: f64,
    pub terrain_grid_n: usize,
    pub terrain_spacing_m: f64,
    pub terrain_elev_min_m: f64,
    pub terrain_elev_max_m: f64,
    pub terrain_elev_std_m: f64,
    pub loss_min_db: Option<f64>,
    pub loss_max_db: Option<f64>,
}

#[derive(Serialize)]
pub struct CoverageRadiusResponse {
    pub max_radius_km: f64,
    pub min_radius_km: f64,
    pub avg_radius_km: f64,
    pub per_bearing: Vec<(f64, f64)>,
}
