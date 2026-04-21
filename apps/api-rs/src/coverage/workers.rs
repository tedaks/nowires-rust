use crate::itm_bridge::itm_p2p_loss;
use crate::signal_levels::sample_line_from_grid;
use crate::terrain::build_pfl;

const ITM_LOSS_UPPER_BOUND: f64 = 400.0;

pub struct ITMWorkerResult {
    pub i: usize,
    pub j: usize,
    pub loss_db: f64,
    pub prx: f64,
}

pub fn itm_worker(
    grid_data: &[Vec<f32>],
    grid_meta: &GridMeta,
    args: &ITMWorkerArgs,
) -> Option<ITMWorkerResult> {
    let elevs = sample_line_from_grid(
        grid_data,
        grid_meta.min_lat,
        grid_meta.max_lat,
        grid_meta.min_lon,
        grid_meta.max_lon,
        grid_meta.n_lat,
        grid_meta.n_lon,
        grid_meta.tx_lat,
        grid_meta.tx_lon,
        args.target_lat,
        args.target_lon,
        args.n_pts,
    );
    let elevs_f64: Vec<f64> = elevs.iter().map(|&v| v as f64).collect();
    let pfl = build_pfl(&elevs_f64, args.step_m);

    let res = itm_p2p_loss(
        args.tx_h_m,
        args.rx_h_m,
        &pfl,
        args.climate,
        args.n0,
        args.f_mhz,
        args.polarization,
        args.epsilon,
        args.sigma,
        12,
        args.time_pct,
        args.location_pct,
        args.situation_pct,
    );

    if !res.loss_db.is_finite() || res.loss_db > ITM_LOSS_UPPER_BOUND {
        return None;
    }
    let prx = args.eirp_dbm + args.ant_gain_adj + args.rx_gain_dbi - res.loss_db;
    Some(ITMWorkerResult {
        i: args.i,
        j: args.j,
        loss_db: res.loss_db,
        prx,
    })
}

pub struct GridMeta {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
    pub n_lat: usize,
    pub n_lon: usize,
    pub tx_lat: f64,
    pub tx_lon: f64,
}

pub struct ITMWorkerArgs {
    pub i: usize,
    pub j: usize,
    pub target_lat: f64,
    pub target_lon: f64,
    pub step_m: f64,
    pub n_pts: usize,
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
    pub eirp_dbm: f64,
    pub ant_gain_adj: f64,
    pub rx_gain_dbi: f64,
}
