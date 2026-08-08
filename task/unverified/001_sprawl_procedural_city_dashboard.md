# SPRAWL - Procedural City Generation Dashboard

## Execution State

- **Executor Type:** any
- **filed_by:** i4@wbox.pro
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ❓ (Unverified)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Ship a working Rust→Wasm procedural city-generation engine — the `sprawl` crate — exposing a `wasm_bridge` API (`init`, `step_terrain`, `step_hydrology`, `step_hubs`, `step_traffic`, `step_parcels`, `get_render_buffer`, `set_parameter`, `get_stats_json`) so a browser dashboard can drive terrain generation, hydrology, hub/traffic simulation, road/parcel subdivision, and segmentation-mask export — built by importing and orchestrating existing cgtools crates (`tiles_tools`, `minwebgl`, `ndarray_cg`, `primitive_generation`, `line_tools`, and others — see Reuse Analysis below) rather than duplicating spatial, rendering, or pathfinding logic that already exists in the workspace. Motivated by there being no procedural-city tool anywhere in cgtools today. Testable via the Acceptance Criteria and Test Matrix below (e.g., `sprawl` compiles to `wasm32-unknown-unknown`; A* simulation with 1,000+ agents completes in <2s; 60 FPS render at 2048x2048).

**Scoped: not yet single-deliverable.** This goal currently bundles all 5 Development Milestones (Wasm bridge/canvas; terrain/hydrology/shoreline; hub placement/traffic/roads/bridges; parcel subdivision/labels; AI integration/segmentation/polish) as one deliverable — see `## Verification Findings` below, which is exactly the open question this normalization pass leaves unresolved (tracked as `Q-01` in `task/decisions.md`).

## In Scope

- New crate `sprawl` at `module/helper/sprawl`, with internal modules: `terrain`, `hydrology`, `shoreline`, `hubs`, `traffic`, `roads`, `bridges`, `parcels`, `labels`, `segmentation`, `wasm_bridge`
- `wasm_bridge` public API: `init(seed, width, height)`, `step_terrain()`, `step_hydrology()`, `step_hubs()`, `step_traffic(agent_count)`, `step_parcels()`, `get_render_buffer() -> *const u8`, `set_parameter(name, value)`, `get_stats_json() -> JsValue`
- New workspace dependencies registered in root `Cargo.toml` `[workspace.dependencies]`: `noise`, `geo`, `petgraph`, `rstar`
- All deliverables described under Development Milestones 1-5 below: Wasm bridge/canvas pipeline; terrain/hydrology/shoreline generation; hub placement/traffic simulation/road network/bridges; parcel subdivision/building footprints; AI-integration hooks (toponymy JSON export, segmentation mask export)/label relaxation/dashboard UI wiring

## Out of Scope

- The frontend application itself (React/Svelte, TypeScript dashboard shell) — lives outside the Rust workspace; this task delivers only the `wasm-bindgen` export surface it consumes
- WebGPU rendering backend (`minwebgpu`) — explicitly a stretch goal in the original spec, deferred
- 3D satellite view via the `renderer` crate — listed as an optional use of an existing crate, not a required deliverable
- Hydraulic erosion simulation in `sprawl::hydrology` — explicitly marked "optional stretch" under Milestone 2
- Actual calls to external AI APIs (Gemini/OpenAI, Stable Diffusion/ControlNet) — orchestrated by the (out-of-scope) frontend; this task only produces the JSON/PNG payloads they would consume

## Description

Implement "SPRAWL", a high-performance web-based procedural city generation dashboard. The application simulates urban growth from terrain generation through traffic routing, culminating in an AI-generated photorealistic satellite map. The simulation engine runs entirely as Rust compiled to WebAssembly via `wasm-bindgen`, handling all math, procedural generation, graph routing, and spatial queries at near-native speed. A frontend UI (React or Svelte, TypeScript) provides the dashboard shell, sidebars, and state management. Rendering targets the HTML5 Canvas / WebGL pipeline driven by the Wasm module's state. An external AI layer (Gemini/OpenAI for text, Stable Diffusion/ControlNet for imagery) is orchestrated by the frontend.

The project must maximally reuse existing cgtools workspace crates and dependencies. Where functionality is missing, new crates are created within the `module/` hierarchy following established patterns (`mod_interface`, `error_tools`, 2-space indent, `tests/` directory, etc.).

---

## Reuse Analysis: Existing cgtools Crates

### Directly Reusable (Strong Match)

| Crate | SPRAWL Use | Capability |
|-------|-----------|------------|
| `tiles_tools` | Grid storage, A* pathfinding, flow fields, FOV, ECS, spatial queries | Quadtree (`spatial.rs`, 672 lines), A* pathfinding, Grid2D storage, HECS ECS, flow fields, field-of-view |
| `minwebgl` | Primary WebGL rendering pipeline for the center canvas | Complete WebGL 2.0: shaders, buffers, textures, VAOs, UBOs, framebuffers, drawbuffers, exec_loop |
| `ndarray_cg` | 2D vector math, matrix operations, transformations | Vectors (Vec2/Vec3/Vec4), matrices (Mat2x2 through Mat4x4), quaternions, approximate equality |
| `mdmath_core` | Core arithmetic, float operations | Vector arithmetic, float traits, index operations |
| `primitive_generation` | Geometry generation for building footprints, road meshes | Contour-to-geometry, text mesh generation, curve tessellation |
| `line_tools` | Road and river rendering with variable width | 2D/3D line rendering, line caps, joins, variable width |
| `animation` | Smooth transitions, growth simulation, UI animations | Easing functions (cubic, etc.), interpolation, timeline |
| `canvas_renderer` | 2D overlay rendering, minimap, PiP window | Canvas abstraction with shader pipeline |
| `browser_input` | Pan, zoom, mouse/keyboard/touch interaction | Mouse events, keyboard events, touch, wheel (zoom) |
| `browser_log` | Debug logging in browser console | Console log/warn/error via web-sys |
| `renderer` | PBR rendering pipeline (optional 3D satellite view) | WebGL-based 3D renderer with material system |
| `mingl` | Foundation graphics abstractions, error handling | Buffer management, data types, camera controls, OBJ loading |

### Available via Workspace Dependencies

| Dependency | SPRAWL Use | Status |
|-----------|-----------|--------|
| `pathfinding` (4.14.0) | A* and Dijkstra for traffic routing | Already used by `tiles_tools` |
| `wfc` (0.10) + `wfc_image` (0.12) | Wave Function Collapse for district/parcel patterns | In workspace deps, example exists (`hexagonal_grid`) |
| `csgrs` (0.20.1, `delaunay` feature) | Delaunay triangulation → Voronoi dual for parcel subdivision | In workspace deps with `delaunay` feature enabled |
| `rayon` (1.10) | Parallel terrain generation, agent simulation | In workspace deps |
| `rand` (0.9.2) + `fastrand` (2.3.0) | RNG for procedural generation | In workspace deps |
| `serde` (1.0) + `serde_json` (1.0) | Map serialization, AI API payloads | In workspace deps |
| `image` (0.25.6) | Segmentation mask export, texture handling | In workspace deps |
| `uuid` (1.17.0) | Unique entity IDs for buildings, roads | In workspace deps |
| `ndarray` (0.16.1) | 2D elevation/moisture arrays (heightmap tensor) | In workspace deps |
| `wasm-bindgen` (0.2.100) | Rust ↔ JS bridge | In workspace deps |
| `web-sys` (0.3.77) | Browser APIs (Canvas, fetch for AI calls) | In workspace deps |
| `base64` (0.22.1) | Encoding segmentation mask for AI API | In workspace deps |
| `itertools` (0.14.0) | Iterator combinators for generation algorithms | In workspace deps |
| `bytemuck` (1.23) | Zero-copy buffer uploads to GPU | In workspace deps |

### Missing Dependencies (Must Add to Workspace)

| Dependency | SPRAWL Use | Suggested Version |
|-----------|-----------|-------------------|
| `noise` | Simplex/Perlin/Worley noise for terrain elevation and moisture maps | `0.9` |
| `geo` | Polygon operations (union, intersection, difference, point-in-polygon) | `0.29` |
| `geo-booleanop` or `geo` built-in | Boolean operations on shoreline/water/land polygons | via `geo` |
| `petgraph` | Explicit graph data structure for road network (complement to A*) | `0.7` |
| `rstar` | R-tree spatial index for "which buildings near this road" queries (complement to quadtree) | `0.12` |

### New Crates to Create

| Crate | Location | Responsibility |
|-------|----------|----------------|
| `sprawl` | `module/helper/sprawl` | Orchestrate procedural city pipeline (terrain → hubs → traffic → parcels → render) |

The `sprawl` crate is the single new entry point. It imports from existing crates and implements the SPRAWL-specific algorithms as internal modules:

**Internal modules within `sprawl`:**

| Module | Responsibility |
|--------|----------------|
| `terrain` | Generate elevation/moisture maps via layered noise, classify biomes |
| `hydrology` | Simulate hydraulic erosion, carve rivers, pool lakes |
| `shoreline` | Marching squares vectorization of noise→polygon boundaries |
| `hubs` | Poisson disk sampling for infrastructure hub placement |
| `traffic` | Spawn agents, pathfind between hubs, accumulate heatmap tensor |
| `roads` | Convert traffic heatmap to road hierarchy (highway/arterial/local) |
| `bridges` | Detect road-water intersections, compute shortest perpendicular crossing |
| `parcels` | Subdivide inter-road polygons into city blocks and building footprints |
| `labels` | Force-directed / simulated annealing label relaxation |
| `segmentation` | Render color-coded segmentation mask for AI image generation |
| `wasm_bridge` | `#[wasm_bindgen]` exports, JS ↔ Rust data transfer, canvas drawing API |

---

## Technical Specification

### Phase 1: Core Mathematics and Spatial Logic

**2D Vector and Math Library**
- Reuse: `ndarray_cg` for Vec2/Vec3/Mat3x3/Mat4x4 operations
- Reuse: `mdmath_core` for float arithmetic traits
- No new math needed; existing SIMD-friendly ndarray backend is sufficient

**Spatial Indexing**
- Reuse: `tiles_tools::spatial` quadtree for broad-phase spatial queries (region queries, circle queries)
- Add: `rstar` crate for R-tree index ("which buildings are near this road segment" queries with bounding-box overlap)
- Both structures coexist: quadtree for dynamic agent simulation, R-tree for static geometry queries

**Geometry and Spatial Culling**
- Add: `geo` crate for polygon boolean operations (union, intersection, difference)
- Reuse: `primitive_generation` for tessellating polygons into renderable meshes
- Reuse: `csgrs` (Delaunay feature) for triangulation of complex polygons
- Implement: point-in-polygon via `geo::Contains` trait (ensure buildings dont overlap water)

### Phase 2: Terrain and Environment Generation

**Elevation and Moisture Maps**
- Add: `noise` crate for layered Simplex/Perlin noise
- Storage: `ndarray::Array2<f64>` for elevation grid, moisture grid
- Biome classification: simple thresholds on (elevation, moisture) → enum { Water, Sand, Grass, Rock, Snow }
- Reuse: `rayon` for parallel row-wise noise evaluation

**Hydraulic River Generation**
- Implement in `sprawl::hydrology`: particle-based erosion simulation
- Drop virtual raindrops on elevation map, simulate downhill flow using gradient descent
- Carve paths by lowering elevation along particle trajectory
- Pool detection when particle enters local minimum (forms lakes)
- Uses `ndarray::Array2` directly, `fastrand` for raindrop placement

**Shoreline Discovery (Vectorization)**
- Implement in `sprawl::shoreline`: marching squares algorithm
- Input: binary grid (elevation < sea_level → water, else land)
- Output: `Vec<Vec<(f64, f64)>>` closed polygon rings (shoreline contours)
- Convert to `geo::Polygon` for boolean operations downstream
- Smooth contours via Catmull-Rom or Chaikin subdivision (reuse `primitive_generation` curve utilities)

### Phase 3: Urban Planning and Infrastructure

**Infrastructure Seeding (Hubs)**
- Implement in `sprawl::hubs`: Poisson disk sampling
- Bridson's algorithm for O(n) sample generation on the terrain grid
- Suitability heuristic: prefer flat terrain (low elevation gradient) near water polygons (coastline proximity)
- Reuse: `tiles_tools::spatial` quadtree for neighbor rejection during sampling
- Output: `Vec<Hub>` with position, type (port, industrial, residential, commercial)

**Traffic Analysis and Arterial Discovery**
- Implement in `sprawl::traffic`: agent-based pathfinding simulation
- Build navigation graph: grid nodes with terrain-cost edges (steep = expensive, water = impassable)
- Reuse: `pathfinding::directed::astar` for individual agent routing between hub pairs
- Add: `petgraph::Graph` for explicit road network graph representation
- Spawn N agents (1,000–10,000), pathfind each between random hub pairs
- Accumulate path segments into `ndarray::Array2<u32>` heatmap
- Classify: count > threshold_high → highway, count > threshold_med → arterial, else → local road
- Reuse: `rayon` for parallel agent batches

**Road Network Generation**
- Implement in `sprawl::roads`: convert heatmap to vector road segments
- Trace connected components of heatmap cells above threshold
- Simplify via Douglas-Peucker (available in `geo` crate)
- Assign road width based on classification (highway=wide, local=narrow)
- Reuse: `line_tools` for rendering roads with proper width, caps, joins

**Bridge Builders**
- Implement in `sprawl::bridges`: road-water intersection detection
- For each road segment crossing a water polygon: compute perpendicular crossing
- Find shortest orthogonal distance across water polygon using `geo::line_intersection`
- Generate bridge geometry: straight span with on/off ramps

### Phase 4: City Blocks and Labeling

**Polygon Subdivision (Parcels)**
- Implement in `sprawl::parcels`: recursive polygon subdivision
- Input: closed polygons formed by road network
- Algorithm: OBB (Oriented Bounding Box) subdivision
  1. Compute OBB of polygon
  2. Split along longest axis
  3. Recurse until area < min_parcel_size
- Alternative: Voronoi relaxation using `csgrs` Delaunay → dual Voronoi
- Reuse: `wfc` for procedural district type assignment (commercial, residential, park, industrial)
- Output: `Vec<Parcel>` with polygon, type, area

**Label Relaxation (Anti-Collision)**
- Implement in `sprawl::labels`: force-directed label placement
- Each label = physical body with repulsive force against other labels and road intersections
- Simple force simulation: iterate N steps, apply spring forces, converge to non-overlapping positions
- Constraint: labels must remain within their feature's bounding box
- Reuse: `tiles_tools::spatial` quadtree for efficient neighbor lookup during force calculation

### Phase 5: AI Integrations (Frontend/API Layer)

**Toponymy Engine (Naming)**
- Frontend extracts feature coordinates and types from Wasm state
- Sends structured JSON to Gemini/OpenAI API with prompt template:
  `"Name these map features: Hub at coast → [port city name], Long winding road → [road name], ..."`
- Wasm module exposes `get_features_json() -> JsValue` via `wasm_bridge`

**Orbital Imaging (Satellite Render)**
- Implement in `sprawl::segmentation`: render color-coded mask
- Render to offscreen `ndarray::Array2<[u8; 4]>` RGBA buffer:
  - Water: `#0000FF`, Roads: `#808080`, Buildings: `#FFFFFF`, Parks: `#00FF00`
- Reuse: `image` crate to encode as PNG
- Reuse: `base64` crate to encode for API payload
- Frontend sends mask to Stable Diffusion ControlNet API
- Result displayed in Picture-in-Picture overlay

### UI and Rendering Architecture

**Center Canvas (60 FPS)**
- Reuse: `minwebgl` for WebGL rendering pipeline
- Reuse: `browser_input` for pan/zoom (mouse drag, wheel zoom, pinch)
- Reuse: `animation` for smooth transitions between generation phases
- Render layers: terrain (textured quad), water (blue overlay), roads (`line_tools`), buildings (instanced quads), labels (text), hubs (icons)
- Reuse: `minwebgl::exec_loop` for requestAnimationFrame render loop

**Wasm Bridge**
- `wasm_bridge` module exposes:
  - `init(seed: u32, width: u32, height: u32)` - initialize simulation
  - `step_terrain()`, `step_hydrology()`, `step_hubs()`, `step_traffic(agent_count: u32)`, `step_parcels()` - phase steps
  - `get_render_buffer() -> *const u8` - zero-copy buffer pointer for WebGL upload
  - `set_parameter(name: &str, value: f64)` - live parameter adjustment from UI sliders
  - `get_stats_json() -> JsValue` - current simulation statistics

---

## Development Milestones

### Milestone 1: Wasm Bridge and Canvas
- Set up `sprawl` crate with `wasm-bindgen` exports
- Prove Rust → Wasm → Canvas pipeline: draw colored rectangles from Rust memory
- Wire `browser_input` for pan/zoom on the canvas
- Integrate `minwebgl::exec_loop` for 60 FPS render loop

### Milestone 2: Terrain and Water
- Implement noise-based elevation + moisture generation
- Implement marching squares for shoreline vectorization
- Render terrain with biome colors, water polygons with fill
- Implement hydraulic erosion for river carving (optional stretch)

### Milestone 3: Graph and Traffic
- Implement Poisson disk sampling for hub placement
- Build navigation graph from terrain grid
- Run A* traffic simulation with configurable agent count
- Render traffic heatmap, classify road hierarchy
- Render roads using `line_tools` with width based on classification

### Milestone 4: Geometry and Subdivision
- Implement polygon subdivision for city blocks
- Generate building footprints within parcels
- Render buildings as instanced geometry
- Implement bridge detection and rendering

### Milestone 5: AI and Polish
- Connect Gemini/OpenAI for toponymy (naming) via frontend
- Implement segmentation mask export
- Integrate ControlNet API for satellite imagery
- Implement label relaxation algorithm
- Wire dashboard UI: left panel sliders, bottom timeline, PiP overlay
- Apply dark theme styling (`#0B131E` background, neon cyan/magenta overlays)

---

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `verb/test` passes with zero failures and zero warnings
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`
-   All Rust code uses 2-space indentation, `mod_interface` pattern, `error_tools` exclusively
-   All tests in `tests/` directories, no mocking, no `cargo fmt`
-   New `sprawl` crate follows workspace conventions: `mod_interface`, `former` builders, feature-gated modules
-   Maximize reuse of existing workspace crates; no reimplementation of existing functionality
-   New workspace dependencies (`noise`, `geo`, `petgraph`, `rstar`) added to root `Cargo.toml` `[workspace.dependencies]`
-   Frontend code (React/Svelte) lives outside the Rust workspace, interfacing only via `wasm-bindgen` exports
-   All rendering goes through `minwebgl` (or `minwebgpu` for WebGPU backend stretch goal)

## Test Matrix

*(Required for tasks that produce tests. Write before writing any test code.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Compile `sprawl` crate | target `wasm32-unknown-unknown` | Compiles without errors |
| T02 | Generate terrain from elevation+moisture noise grid | default seed | Produces visually distinct biomes (Water/Sand/Grass/Rock/Snow) |
| T03 | Run marching squares on binary terrain grid (sea_level threshold) | default terrain | Produces closed vector polygon rings |
| T04 | Run Poisson disk sampling (Bridson's algorithm) on terrain | coastline-suitability bias enabled | Evenly-spaced hubs biased toward flat, coastal terrain |
| T05 | Run A* traffic simulation with 1,000+ agents | wasm release build | Completes in <2 seconds |
| T06 | Classify traffic heatmap cells against thresholds | threshold_high / threshold_med | Produces highway/arterial/local road hierarchy |
| T07 | Subdivide polygons formed by the road network | OBB subdivision | Produces non-overlapping city-block parcels |
| T08 | Render segmentation mask for all feature types | color-coded RGBA buffer | Correct color codes per feature type (water/roads/buildings/parks) |
| T09 | Render 2048x2048 terrain with pan/zoom active | Canvas/WebGL pipeline | Sustains 60 FPS |
| T10 | Run full existing workspace test suite after `sprawl` is added | Level 3 (nextest + doctests + clippy) | All pre-existing tests continue to pass |
| T11 | Run clippy across workspace after `sprawl` is added | `--all-targets --all-features -- -D warnings` | Zero new clippy warnings |
| T12 | Run per-phase integration tests for `sprawl` | terrain/hydrology/hubs/traffic/parcels/render phases | Each generation phase has passing integration test coverage |

## Acceptance Criteria

- `sprawl` crate compiles to wasm32-unknown-unknown without errors
- Terrain generation produces visually distinct biomes from noise (elevation + moisture grid)
- Marching squares produces closed vector polygons from binary terrain data
- Poisson disk sampling generates evenly-spaced hubs with terrain suitability bias
- A* traffic simulation with 1,000+ agents completes in <2 seconds (wasm, release mode)
- Road hierarchy (highway/arterial/local) derived from traffic heatmap thresholds
- Polygon subdivision generates city blocks between road segments
- Segmentation mask renders correct color codes for all feature types
- 60 FPS canvas rendering with pan/zoom for a 2048x2048 terrain
- All existing workspace tests continue to pass (Level 3: nextest + doctests + clippy)
- No new clippy warnings introduced
- `sprawl` crate has integration tests covering each generation phase
- Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Wasm bridge & rendering**
- [ ] C1 — Does `sprawl` compile to `wasm32-unknown-unknown` without errors?
- [ ] C2 — Does the `wasm_bridge` module expose `init`, `step_terrain`, `step_hydrology`, `step_hubs`, `step_traffic`, `step_parcels`, `get_render_buffer`, `set_parameter`, `get_stats_json`?

**Terrain & hydrology**
- [ ] C3 — Does terrain generation produce visually distinct biomes from the elevation+moisture grid?
- [ ] C4 — Does marching squares produce closed vector polygons from binary terrain data?

**Traffic & roads**
- [ ] C5 — Does Poisson disk sampling generate evenly-spaced hubs with terrain-suitability bias?
- [ ] C6 — Is road hierarchy (highway/arterial/local) derived from traffic heatmap thresholds?

**Parcels & imagery**
- [ ] C7 — Does polygon subdivision generate city blocks between road segments?
- [ ] C8 — Does the segmentation mask render correct color codes for all feature types?

**Out of Scope confirmation**
- [ ] C9 — Is the React/Svelte frontend application code absent from this crate (interfacing only via `wasm-bindgen` exports)?
- [ ] C10 — Is a WebGPU (`minwebgpu`) rendering backend absent (deferred stretch goal)?
- [ ] C11 — Is the 3D satellite view (`renderer` crate) absent (deferred stretch goal)?
- [ ] C12 — Is hydraulic erosion simulation absent from `sprawl::hydrology` (deferred stretch goal)?

### Measurements

- [ ] M1 — Traffic simulation throughput: `1,000+ agent A* run, wasm release build` → completes in <2s (was: not yet implemented)
- [ ] M2 — Render frame rate: `2048x2048 terrain, pan/zoom active` → sustains 60 FPS (was: not yet implemented)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures (Level 3: nextest + doctests + clippy)
- [ ] I2 — compiler clean: `cargo clippy --all-targets --all-features -- -D warnings` → 0 new warnings

### Anti-faking checks

- [ ] AF1 — per-phase integration coverage: `grep -rn "#\[test\]" module/helper/sprawl/tests/` → at least one test per generation phase (terrain, hydrology, hubs, traffic, parcels, render), not a single monolithic smoke test

## Verification Findings

Readiness Verification Gate (`tsk.rulebook.md § Task File : Readiness Verification Gate`) — Tier 2 Dual-Role Self-Check, Round 1, self-administered during normalization:

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | In Scope/Out of Scope were both structurally absent before this pass | Both populated above; meaningful observable outcome; Scope Sizing Gate's 3 sub-checks all pass |
| D2 | MOST Goal Quality | — | 🔴 | Motivated/Observable/Testable all hold, but **Scoped** fails: the Goal spans all 5 Development Milestones bundled as one deliverable, not a single bounded deliverable | Not yet applied — open decision `Q-01` in `task/decisions.md` (split into 5 tasks / narrow to one milestone / extract to a Governing Plan) |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skipping this task means cgtools has no procedural-city tool; detail level in the spec signals committed intent, not speculative scaffolding | — |
| D4 | Implementation Readiness | — | 🟢 | Delivery Requirements are concrete; Test Matrix populated (12 rows, 1:1 with Acceptance Criteria); Acceptance Criteria specific and verifiable | — |
| D5 | Execution Scope | — | 🟢 | All deliverable paths (`module/helper/sprawl`, root `Cargo.toml`) resolve inside this repository | — |
| D6 | Crate Scope Unity | — | 🟢 | All functional deliverables resolve inside the one new `sprawl` crate; the root `Cargo.toml` touch is mechanical (registering the new crate + its deps), not a second crate's functional scope | — |
| D7 | Crate Locality | — | 🟢 | All generation logic (terrain, hydrology, traffic, parcels, etc.) targets the new leaf crate `sprawl` itself, not an aggregator | — |
| D8 | Crate Single Responsibility | — | 🟢 | `sprawl`'s responsibility states in one sentence without "and": orchestrate the procedural city-generation pipeline (terrain → hubs → traffic → parcels → render); internal modules are pipeline stages, not unrelated concerns | — |
| **Total** | | — | 🔴 | 1 open (D2) | 0/1 |

**Aggregate verdict:** FAIL (D2 only) — task remains ❓ Unverified. Re-run this gate once `Q-01` (`task/decisions.md`) is resolved and the Goal/In Scope/Out of Scope above are updated to match.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-08 12:40:02]** `FILED` — Backfilled: original filing date not recorded (task predates `## History` section adoption; git log unavailable this session per user instruction — see `governance/maav.rulebook.md`-adjacent session constraint). Task retroactively attributed to i4@wbox.pro based on repository authorship context. Goal (as originally filed): implement the SPRAWL procedural city generation dashboard.
- **[2026-08-08 12:40:03]** `NOTE` — Normalized into canonical `tsk.rulebook.md` structure: added `## Execution State`, `## Goal`, `## In Scope`, `## Out of Scope`, `## Delivery Requirements`, `## Test Matrix`, `## Verification`, `## Verification Findings`, `## History`; corrected `## Requirements` rulebook-discovery citation from non-canonical `prompt .rulebooks.relevant` to `kbase .rulebooks`; moved from `task/` root to `task/unverified/`. Pre-existing content (`## Description`, `## Reuse Analysis`, `## Technical Specification`, `## Development Milestones`, `## Acceptance Criteria`) preserved verbatim in place. Readiness Verification Gate run: D2 (MOST Goal Quality — Scoped) FAILs on 5-milestone bundling; task left at ❓ Unverified pending `Q-01` (`task/decisions.md`).
