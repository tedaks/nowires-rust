# NoWires

A radio propagation analysis system powered by NTIA's Irregular Terrain Model (ITM).

NoWires computes point-to-point path loss, terrain profiles with Fresnel zone analysis, and area coverage predictions. It combines a Rust/Axum backend with GDAL-powered terrain data and an interactive MapLibre frontend.

## Features

- **Point-to-Point Analysis**: Click two points on the map for TX and RX. The app plots a terrain profile with line-of-sight, 1st Fresnel zone, and 60% Fresnel zone, and reports ITM basic transmission loss with link budget.
- **Area Coverage**: Place a transmitter and generate a color-coded coverage overlay showing signal strength over the area, with adjustable grid resolution up to 512×512.
- **Coverage Radius**: Per-bearing radius estimation showing maximum, minimum, and average coverage distance.
- **Multi-Site Comparison**: Save coverage results as named sites and overlay multiple transmitters with adjustable opacity.
- **Directional Antennas**: Support for omnidirectional and directional antenna patterns with configurable azimuth and beamwidth.

## Setup

### Prerequisites

- Rust 1.75+ (with Cargo)
- Node.js 20+
- npm
- GDAL 3.x (for elevation tile processing)

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Environment

Copy `.env.example` to `.env` and configure:

```env
# Elevation data directories (optional — will use API fallback if not set)
SRTM1_TILES_DIR=/path/to/srtm1/tiles
GLO30_TILES_DIR=/path/to/glo30/tiles

# Optional land cover data for additional analysis
LANDCOVER_DIR=/path/to/landcover/tiles

# CORS origins for production (comma-separated). If not set, falls back to DEV_ORIGINS.
CORS_ORIGINS=https://nowires.example.com

# Elevation cache directory (defaults to data/elev_cache/)
ELEV_CACHE_DIR=/path/to/cache
```

For frontend development, create `apps/web/.env.local`:

```env
BACKEND_URL=http://127.0.0.1:8000
DEV_ORIGINS=http://192.168.1.100:3000
```

### 3. Run the Backend

```bash
npm run dev:api
```

### 4. Run the Frontend

```bash
npm run dev:web
```

Open http://localhost:3000 in your browser.

## Elevation Data

NoWires fetches terrain elevation from multiple sources in priority order:

1. **GLO30** (Copernicus 30m DEM) — highest priority
2. **SRTM1** (1-arcsecond / 30m) — GeoTIFF tiles
3. GDAL fallback (automatic)

For best performance, download and configure local GLO30 or SRTM1 tiles. Tiles must be organized in the standard directory structure (e.g., `N14/E120.tif` for GLO30).

Elevation grids are cached on disk with deterministic filenames and atomic writes for concurrency safety.

## Architecture

```
nowires/
├── apps/
│   ├── api-rs/                  # Rust/Axum backend
│   │   └── src/
│   │       ├── main.rs          # Server, CORS, body limit, semaphore
│   │       ├── routes/          # API endpoint handlers
│   │       │   ├── p2p.rs
│   │       │   ├── coverage.rs
│   │       │   └── coverage_radius.rs
│   │       ├── models.rs        # Request structs + validation
│   │       ├── models/response.rs  # Response structs
│   │       ├── itm_bridge.rs    # rustitm wrapper + ITMParams
│   │       ├── rounding.rs      # round1/round2/round3 helpers
│   │       ├── terrain.rs       # Haversine, bearing, profile, PFL
│   │       ├── elevation/       # GDAL GeoTIFF elevation grid + cache
│   │       ├── coverage/        # GridMeta, ITM workers, PNG render, radius
│   │       ├── fresnel.rs       # Fresnel profile + coverage coloring
│   │       ├── signal_levels.rs # dBm thresholds, colors, sampling
│   │       └── antenna.rs       # Antenna gain pattern
│   └── web/                    # Next.js 16 + TypeScript frontend
│       └── src/
│           ├── app/            # Routes and error boundary
│           ├── components/     # MapLibre map, P2P panel, Coverage panel
│           └── lib/            # API client, types, utilities
├── data/                       # Runtime cache (gitignored)
├── .github/workflows/ci.yml   # CI pipeline
└── LICENSE.md
```

## Server Configuration

- **Body size limit**: 2 MB max request body
- **Concurrency limit**: 4 simultaneous expensive requests (coverage, radius)
- **Port**: 8000 (exits with error message on bind failure)
- **CORS**: `CORS_ORIGINS` env var (production), falls back to `DEV_ORIGINS`, then localhost defaults

## Testing

```bash
# Frontend unit tests (vitest)
npm --workspace apps/web run test

# Backend tests (rust unit tests)
cd apps/api-rs && cargo test

# Full CI
npm run lint && npm run typecheck && npm run build:web
cd apps/api-rs && cargo clippy && cargo fmt --check && cargo test
```

## Performance

| Grid Size | Cold Start | Cached  |
|-----------|-----------|---------|
| 192×192   | ~2.3s     | <10ms   |
| 384×384   | ~8s       | <10ms   |

Key optimizations:
- Bulk GDAL GeoTIFF reads (entire raster loaded once per tile)
- Rayon parallelism for coverage grid computation
- Capped profile lengths for distant pixels (configurable, default max 75)
- Deterministic file-based elevation cache with atomic writes
- Request body size limit (2 MB) and concurrency semaphore (4 permits)

## Credits

NoWires uses the [rustitm](https://github.com/tedaks/rustitm) library for NTIA Irregular Terrain Model calculations and the [gdal](https://gdal.org/) crate for terrain elevation processing.
