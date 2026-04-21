# NoWires — Agent Instructions

## Project Overview
Radio propagation analysis system using NTIA Irregular Terrain Model (ITM).
- **Backend**: Rust/Axum at `apps/api-rs/` with rustitm + GDAL
- **Frontend**: Next.js 16 + TypeScript + Tailwind + MapLibre at `apps/web/`

## Commands

### Frontend
```bash
npm run dev:web          # Start Next.js dev server
npm run build:web        # Production build
npm run lint             # ESLint
npm run typecheck        # TypeScript check (tsc --noEmit)
npm run test:watch       # vitest watch mode
npm run test:e2e         # Playwright E2E tests
```

### Backend
```bash
npm run dev:api          # Start Axum dev server (cargo run)
cd apps/api-rs && cargo build           # Build
cd apps/api-rs && cargo clippy          # Lint
cd apps/api-rs && cargo fmt --check     # Format check
cd apps/api-rs && cargo test            # Run tests
```

### Full CI
```bash
npm run lint && npm run typecheck && npm run build:web
cd apps/api-rs && cargo clippy && cargo fmt --check && cargo test
```

## Architecture
- `apps/api-rs/src/main.rs` — Axum server, CORS, body size limit, semaphore concurrency limit
- `apps/api-rs/src/routes/p2p.rs` — POST /api/p2p handler
- `apps/api-rs/src/routes/coverage.rs` — POST /api/coverage handler
- `apps/api-rs/src/routes/coverage_radius.rs` — POST /api/coverage-radius handler
- `apps/api-rs/src/models.rs` — Request serde structs + validation
- `apps/api-rs/src/models/response.rs` — Response serde structs
- `apps/api-rs/src/itm_bridge.rs` — rustitm crate wrapper + `ITMParams` shared struct
- `apps/api-rs/src/rounding.rs` — `round1`/`round2`/`round3` decimal rounding helpers
- `apps/api-rs/src/terrain.rs` — Haversine, bearing, profile, PFL builder
- `apps/api-rs/src/elevation/` — GDAL GeoTIFF elevation grid + deterministic file cache
- `apps/api-rs/src/coverage/workers.rs` — `GridMeta` + ITM coverage pixel workers
- `apps/api-rs/src/coverage/render.rs` — PNG rendering
- `apps/api-rs/src/coverage/radius.rs` — Coverage radius sweep
- `apps/api-rs/src/fresnel.rs` — Fresnel profile + coverage coloring
- `apps/api-rs/src/signal_levels.rs` — dBm thresholds, colors, sampling
- `apps/api-rs/src/antenna.rs` — Antenna gain pattern + `DEFAULT_FRONT_BACK_DB`
- `apps/web/src/components/map/MapView.tsx` — MapLibre GL map component
- `apps/web/src/components/p2p/` — P2P analysis panel + profile chart
- `apps/web/src/components/coverage/` — Coverage panel + sites panel + legend

## Environment Variables
Backend vars are in root `.env` (see `.env.example`). Key ones:
- `SRTM1_TILES_DIR` — SRTM1 GeoTIFF tiles directory
- `GLO30_TILES_DIR` — Copernicus GLO-30 tiles directory
- `LANDCOVER_DIR` — ESA WorldCover land cover tiles directory
- `CORS_ORIGINS` — Comma-separated allowed CORS origins for production (takes precedence over `DEV_ORIGINS`)
- `ELEV_CACHE_DIR` — Elevation cache directory (defaults to `data/elev_cache/`)

Frontend vars are in `apps/web/.env.local` (see `apps/web/.env.local.example`):
- `BACKEND_URL` — Backend URL for Next.js proxy (default: http://127.0.0.1:8000)
- `DEV_ORIGINS` — Comma-separated allowed dev origins (e.g. `http://192.168.2.16:3000`). Next.js requires the `http://` prefix; `next.config.ts` auto-adds it if missing.

## Server Configuration
- **Body size limit**: 2 MB (via `DefaultBodyLimit`)
- **Concurrency limit**: 4 simultaneous expensive requests (via `Arc<Semaphore>`)
- **Port**: 8000 (hardcoded; server exits on bind failure with descriptive message)
- **CORS**: Uses `CORS_ORIGINS` if set, falls back to `DEV_ORIGINS`, then defaults to `localhost:3000,localhost:3001`

## Key Conventions
- Rust: cargo clippy for lint, cargo fmt for format
- TypeScript: ESLint + strict mode, no any types
- Rounding: Use `round1`/`round2`/`round3` from `crate::rounding` instead of inline `(v * N).round() / N`
- Named constants: Magic numbers must be extracted to `const` with docs (e.g., `P2P_STEP_M`, `COVERAGE_MDVAR`)
- Test files: `apps/api-rs/src/**/*.rs` (unit tests), `apps/web/e2e/*.spec.ts`
- API responses use snake_case, TypeScript types mirror the Rust structs

## Strict Rules

### 300-Line File Limit
No source file shall exceed 300 lines. If a file reaches this limit, extract sub-components, utilities, or helpers into separate modules before adding more code. (shadcn/ui auto-generated files in `components/ui/` are exempt.)

### UI Components: shadcn + Tailwind Only
All UI components MUST use shadcn/ui primitives and Tailwind CSS utility classes exclusively. No custom UI primitives, no invented component patterns, no third-party UI libraries beyond shadcn. When a UI element is needed:
1. Check `apps/web/src/components/ui/` for existing shadcn components.
2. If missing, install via `npx shadcn@latest add <component>`.
3. If shadcn doesn't offer it, build it using Tailwind utilities on a native HTML element — never introduce another component library.
4. Never copy-paste shadcn code from external sources with modifications that deviate from the canonical shadcn pattern.
5. Styling must be Tailwind utility classes only — no inline styles, no CSS modules, no styled-components, no creative alternatives.
6. CSS custom properties (e.g., `--site-color`) must be validated before injection — use guards like `isValidHexColor()` or sanitizers like `sanitizeRgba()`.
