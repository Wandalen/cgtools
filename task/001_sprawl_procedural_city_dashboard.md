# SPRAWL - Procedural City Generation Dashboard

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

- All work must strictly adhere to all applicable rulebooks
  (discover via `prompt .rulebooks.relevant`)
- All Rust code uses 2-space indentation, `mod_interface` pattern, `error_tools` exclusively
- All tests in `tests/` directories, no mocking, no `cargo fmt`
- New `sprawl` crate follows workspace conventions: `mod_interface`, `former` builders, feature-gated modules
- Maximize reuse of existing workspace crates; no reimplementation of existing functionality
- New workspace dependencies (`noise`, `geo`, `petgraph`, `rstar`) added to root `Cargo.toml` `[workspace.dependencies]`
- Frontend code (React/Svelte) lives outside the Rust workspace, interfacing only via `wasm-bindgen` exports
- All rendering goes through `minwebgl` (or `minwebgpu` for WebGPU backend stretch goal)

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
