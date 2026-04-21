pub const PROP_MODE_NAMES: [&str; 6] = [
    "Line-of-Sight",
    "Single Horizon Diffraction",
    "Double Horizon Diffraction",
    "Troposcatter",
    "Diffraction LOS Backward",
    "Mixed Path",
];

pub const ITM_LOSS_SENTINEL: f64 = 999.0;

pub struct ITMParams {
    pub tx_h_m: f64,
    pub rx_h_m: f64,
    pub climate: i32,
    pub n0: f64,
    pub f_mhz: f64,
    pub polarization: i32,
    pub epsilon: f64,
    pub sigma: f64,
    pub time_pct: f64,
    pub location_pct: f64,
    pub situation_pct: f64,
}

/// Result from ITM point-to-point propagation computation.
/// Some fields are kept for diagnostic completeness even if not currently used in API responses.
pub struct ITMResult {
    pub loss_db: f64,
    pub mode: i32,
    pub warnings: i32,
    pub d_hzn_tx_m: f64,
    pub d_hzn_rx_m: f64,
    /// Keep for diagnostic use.
    #[allow(dead_code)]
    pub theta_hzn_tx: f64,
    /// Keep for diagnostic use.
    #[allow(dead_code)]
    pub theta_hzn_rx: f64,
    pub h_e_tx_m: f64,
    pub h_e_rx_m: f64,
    /// Keep for diagnostic use.
    #[allow(dead_code)]
    pub n_s: f64,
    pub delta_h_m: f64,
    pub a_ref_db: f64,
    /// Keep for diagnostic use.
    #[allow(dead_code)]
    pub a_fs_db: f64,
    /// Keep for diagnostic use.
    #[allow(dead_code)]
    pub d_km: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn itm_p2p_loss(
    h_tx_meter: f64,
    h_rx_meter: f64,
    profile: &[f64],
    climate: i32,
    n0: f64,
    f_mhz: f64,
    polarization: i32,
    epsilon: f64,
    sigma: f64,
    mdvar: i32,
    time_pct: f64,
    location_pct: f64,
    situation_pct: f64,
) -> ITMResult {
    let rustitm_climate = climate + 1;
    let rustitm_pol = polarization;

    match rustitm::itm_p2p_tls_ex(
        h_tx_meter,
        h_rx_meter,
        profile,
        rustitm_climate,
        n0,
        f_mhz,
        rustitm_pol,
        epsilon,
        sigma,
        mdvar,
        time_pct,
        location_pct,
        situation_pct,
    ) {
        Ok(output) => {
            let iv = &output.inter_values;
            ITMResult {
                loss_db: output.a__db,
                mode: iv.mode,
                warnings: output.warnings,
                d_hzn_tx_m: iv.d_hzn__meter[0],
                d_hzn_rx_m: iv.d_hzn__meter[1],
                theta_hzn_tx: iv.theta_hzn[0],
                theta_hzn_rx: iv.theta_hzn[1],
                h_e_tx_m: iv.h_e__meter[0],
                h_e_rx_m: iv.h_e__meter[1],
                n_s: iv.n_s,
                delta_h_m: iv.delta_h__meter,
                a_ref_db: iv.a_ref__db,
                a_fs_db: iv.a_fs__db,
                d_km: iv.d__km,
            }
        }
        Err(e) => {
            tracing::warn!(
                "ITM computation failed: {:?} (f={} MHz, climate={}, profile_len={})",
                e,
                f_mhz,
                climate,
                profile.len()
            );
            ITMResult {
                loss_db: ITM_LOSS_SENTINEL,
                mode: 0,
                warnings: 1,
                d_hzn_tx_m: 0.0,
                d_hzn_rx_m: 0.0,
                theta_hzn_tx: 0.0,
                theta_hzn_rx: 0.0,
                h_e_tx_m: 0.0,
                h_e_rx_m: 0.0,
                n_s: 0.0,
                delta_h_m: 0.0,
                a_ref_db: 0.0,
                a_fs_db: 0.0,
                d_km: 0.0,
            }
        }
    }
}
