use serde::Serialize;

#[derive(Serialize)]
pub struct P2PResponse {
    pub distance_m: f64,
    pub profile: Vec<ProfilePoint>,
    pub profile_original_count: usize,
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

#[derive(Serialize, Clone, Copy)]
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
    pub profile_step_m: f64,
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
