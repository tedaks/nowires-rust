# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.0] — Initial Release

### Added
- Point-to-point radio link analysis with ITM path loss calculation
- Terrain profile generation with Fresnel zone analysis (1st and 60% zones)
- Area coverage map computation with color-coded signal strength
- Coverage radius estimation per bearing
- Multi-site coverage comparison with overlay support
- Directional antenna pattern support (azimuth, beamwidth, front-back ratio)
- Multiple elevation source support: GLO30, SRTM1, GDAL fallback
- NaN-aware elevation handling with linear interpolation
- Deterministic file-based elevation cache with atomic writes
- Request body size limit (2 MB) and concurrency semaphore (4 permits)
- `CORS_ORIGINS` env var for production CORS configuration
- `ELEV_CACHE_DIR` env var for configurable cache directory
- `ITMParams` shared struct for common ITM computation parameters
- `round1`/`round2`/`round3` decimal rounding helpers
- `profile_original_count` field in P2P response
- `profile_step_m` field in coverage response
- `per_bearing` field in coverage radius response
- vitest unit tests for frontend utilities
- Playwright E2E test infrastructure
- Full CI pipeline (ESLint, TypeScript, vitest, Next.js build, cargo clippy, cargo fmt, Playwright)
- Security audit and code coverage CI jobs

### Changed
- Frontend charting library changed from Plotly to Recharts
- Elevation data now uses NaN instead of 0.0 to represent missing values
- Frontend environment variables moved to `apps/web/.env.local`
- GDAL reads now use bulk rasterband reads instead of per-pixel queries
- Elevation cache keys now use deterministic parameter strings instead of `DefaultHasher`
- Magic numbers extracted to named constants (`P2P_STEP_M`, `COVERAGE_MDVAR`, etc.)
- Inline rounding patterns replaced with `round1`/`round2`/`round3` helpers
- `models.rs` split into `models.rs` (request) and `models/response.rs` (response)
- `GridMeta` and `RadiusGridMeta` unified into single `GridMeta` struct
- `ITM_LOSS_SENTINEL` deduplicated across modules
- TypeScript `CoverageStats` nullable fields (`prx_min_dbm`, etc.) now typed as `number | null`
- API client now validates response content-type before JSON parsing
- CSS custom properties now validated/sanitized before injection
- CI dependency-check uses `continue-on-error` instead of `|| true`

### Fixed
- Corrected ITM time/location/situation parameter scaling
- Fixed elevation source priority chain (GLO30 → SRTM1 → GDAL fallback)
- Fixed climate enum indexing (API uses 0-indexed, ITM uses 1-indexed)
- Replaced `unwrap()` on TCP bind with graceful error handling
- Added `AbortController` cleanup on component unmount to prevent memory leaks
- Added `h_m` upper bound validation (max 10000m)
- Added `ProfilePoint` `Copy` derive for efficiency in downsampling
- Clippy warnings resolved (redundant closures, manual clamp, manual range contains)
