# Falling Frontier → cgtools Port Audit

**Scope:** The Three.js/Vite prototype at `examples/threejs/falling_frontier/` was built to explore a tactical space-scene UI (selectable ships/asteroids, a movable/rotatable gizmo, a shader-driven "tactical grid" with a selection-driven view-zone ribbon that wraps around obstacles, a live dev tuning panel, and a full HTML HUD overlay). This document audits, feature by feature, what already exists in `cgtools` (the Rust/WebGL2 toolkit this prototype would eventually be ported into) versus what would need to be built from scratch.

**Method:** Every source file under `examples/threejs/falling_frontier/src/` was read in full and broken into 76 granular, individually-checkable features. Each feature was then verified against the `cgtools` codebase directly — file paths and code excerpts are cited as evidence rather than general impressions. Nothing below is inferred without a corresponding grep/read.

## Summary

| Status | Count | Meaning |
|---|---|---|
| ✅ Present | 20 | Working equivalent exists in cgtools today |
| 🟡 Partial | 16 | Related infrastructure exists but doesn't fully cover the behavior |
| ❌ Missing | 34 | No equivalent found anywhere in the repo |
| ⚪ N/A | 6 | Content-authoring choice or toolchain difference, not a portable technique |

**Headline finding:** the two biggest, hardest pieces of the prototype — the transform gizmo and the entire tactical-grid shader system (procedural infinite grid, multi-mode fades, and the polyline-based obstacle-occlusion ribbon) — have **no equivalent anywhere in cgtools** and would be built entirely from scratch. The dev tuning panel that made all of this iterable live in the browser is also completely absent, which matters because it's what made the shader work *possible* to tune by feel in the first place (see "Rendering Pipeline Architecture" below).

---

## Rendering Pipeline Architecture (flexibility check)

This section answers the direct question of whether `module/helper/renderer`'s existing render pipeline could simply host a custom shader like the tactical grid, or whether it would fight the port.

**The `Material` trait is nominally extensible, but has exactly one real implementation.**
`module/helper/renderer/src/webgl/material/mod.rs` defines `Material` as a trait with `vertex_shader()`/`fragment_shader()` returning arbitrary GLSL source strings, plus `upload()`/`upload_on_state_change()` hooks for arbitrary per-frame uniform uploads — in principle this is the right shape for a `THREE.ShaderMaterial`-style custom shader. In practice, grepping the entire repo for `impl Material for` turns up exactly **one** implementor: `PbrMaterial` (`material/pbr.rs`). No custom/unlit/grid-style material has ever actually been built against this trait, so its flexibility is unproven outside the one PBR use case it was designed for.

**Any transparent material is forced through a mandatory WBOIT contract.**
This is the concrete blocker. `Renderer::transparent_draw()` (`renderer.rs`) hard-codes Weighted Blended Order-Independent Transparency for every material with `AlphaMode::Blend`: fixed blend state (`blend_func_separate(ONE, ONE, ZERO, ONE_MINUS_SRC_ALPHA)`), and the fragment shader is required to write to **two** specific outputs (a weighted-accumulate color and a revealage value), not a single `gl_FragColor`. Confirmed directly in `src/webgl/shaders/main.frag`:

```glsl
layout( location = 2 ) out vec4 trasnparentA;
layout( location = 3 ) out float transparentB;
...
float a_weight = alpha * alpha_weight( alpha );
trasnparentA = vec4( color * a_weight, alpha );
transparentB = a_weight;
```

The opaque pass, by contrast, never enables `GL_BLEND` at all (`gl.disable(gl::BLEND)` is set once per frame and never re-enabled in `opaque_draw`) — so there is **no simple "just alpha-blend like three.js does" path**. The tactical grid's shader (`transparent: true, depthWrite: false`, straightforward `discard` + `max()`-combined alpha) would either need to be rewritten against this WBOIT weight-function contract, or bypass the `Material`/`Renderer` system entirely.

**In fact, every other custom-shader feature in the repo already bypasses this system.** `line_tools` (which would be the natural fit for the trajectory ribbons and sensor rings) has its own standalone `draw()` method (`line_tools/src/d3/line.rs:366`) with no `impl Material for` anywhere in that crate — it manages its own `ProgramFromSources` and issues its own draw calls, completely outside `Scene`/`Node`/`Renderer`. Post-processing passes (bloom, tonemap, composite) work the same way. So the realistic pattern for porting the grid isn't "write a `Material` impl" — it's "write a fourth standalone rendering subsystem," alongside PBR, `line_tools`, and post-processing, each with its own GL state management that a project-level render loop would have to sequence by hand (share camera matrices, avoid stomping shared GL state, get depth-test ordering right between the grid plane, PBR ship meshes, and line-tool ribbons). Three.js does this sequencing automatically inside one `renderer.render(scene, camera)` call; cgtools does not have that single entry point today.

**Net assessment:** the render pipeline isn't so much "inflexible" as **narrow and single-purpose** — it's a solid PBR-scene renderer with mandatory OIT, not a general-purpose material system. Nothing here blocks the port, but it does mean the grid shader is realistically a bespoke subsystem (like `line_tools` already is), not a drop-in `Material` implementation, and building it will surface the WBOIT-vs-simple-blend mismatch immediately if a `Material` impl is attempted first.

---

## Camera / Controls

| Feature | Status | Evidence / Gap |
|---|---|---|
| Orbit camera with damping | ✅ | `module/min/mingl/src/controls/camera_orbit_controls.rs` — `CameraRotationState.movement_smoothing_enabled` + `movement_decay`, matches three.js `enableDamping`/`dampingFactor`. |
| Min/max zoom distance clamp | ✅ | `CameraZoomState::min_distance_set`/`max_distance_set` in the same file. |
| Polar-angle clamp (camera stays above ground) | ✅ | `CameraRotationState.latitude_range` clamped to `±FRAC_PI_2`; also has an independent `longitude_range` three.js's OrbitControls doesn't offer. |
| Pointer-driven rig (drag-rotate / drag-pan / wheel-zoom / pinch-zoom) | ✅ | `controls_bind_to_input()` — full pointer state machine incl. multi-touch pinch, which the JS example (stock OrbitControls) doesn't customize. |
| Reset camera to default view | ❌ | Grepped for a saved-default/reset API on `CameraOrbitControls` — none; only mutation methods (`rotate`/`pan`/`zoom`) exist. |
| PerspectiveCamera tied to controls | ✅ | `module/helper/renderer/src/webgl/camera.rs::Camera::new()`. |
| Renderer antialias/powerPreference config | 🟡 | Context creation exists (`gl::context::from_canvas`) but there's no single ergonomic constructor-options struct like `new THREE.WebGLRenderer({antialias, powerPreference})` — each example wires context attributes ad hoc. |
| DPR-aware canvas resize | ✅ | `mingl/src/web/canvas.rs::canvas_resize()` — driven by a `ResizeObserver`, actually stronger than the JS demo's plain `window resize` listener. |
| Shadow mapping (PCFSoft-equivalent) | ✅ | `renderer/src/webgl/shadow.rs::ShadowMap`, documented in `docs/feature/003_shadow_mapping.md`. |
| ACES Filmic tonemapping + exposure | ✅ | `renderer/src/webgl/shaders/tonemapping/aces.frag` — comment explicitly notes it matches three.js's ACESFilmicToneMapping. |

## Lighting / Scene Environment

| Feature | Status | Evidence / Gap |
|---|---|---|
| Two-tone hemisphere ambient light | ❌ | Grepped `HemisphereLight` — only hit is inside an IBL/PMREM convolution shader (precomputed spherical-harmonics ambient from an environment map), a different mechanism, not a simple two-color light primitive. |
| Directional sun light w/ shadow frustum | ✅ | Same shadow-mapping infra as above. |
| Visual sun mesh + additive-blended glow shells | ❌ | No multi-shell `AdditiveBlending` glow-sprite technique found. |
| Post-process bloom | ✅ (unused by grid) | `renderer/src/webgl/post_processing/unreal_bloom.rs` — achieves a similar visual goal via a different technique (post-process vs. geometry glow shells). |
| Secondary fill directional light | ✅ | Renderer supports multiple lights generically via the scene graph; no extra evidence needed. |
| Solid-color scene background | ✅ | Trivial (`gl.clear_color`). |

## Selection / Interaction

| Feature | Status | Evidence / Gap |
|---|---|---|
| Raycast-vs-mesh 3D hit-testing | ❌ | `examples/minwebgl/raycaster` is a 2D tile-grid DDA raycaster (Wolfenstein-style walls), not 3D ray-vs-triangle picking. No `Raycaster.intersectObjects` equivalent exists. |
| Click-to-select | 🟡 (different mechanism) | `examples/minwebgl/object_picking` does GPU color-ID picking (render object IDs to an `R32I` framebuffer, read the pixel under the cursor) — a legitimate alternative to CPU raycasting, but architecturally different and not cross-compared against a raycast approach anywhere. |
| pointerdown/pointerup drag-distance-threshold click detection | 🟡 | `browser_input` and `camera_orbit_controls.rs` both already use raw pointer events (never native `click`), so the plumbing is there, but no crate implements the actual "did the pointer move more than N px" threshold check — it's a capability, not a shipped feature. |
| Ignore clicks on DOM UI chrome | ⚪ N/A | No DOM UI layer exists yet to need this exclusion (see HUD section). |
| Walk-up-parent-to-find-selectable-ancestor pattern | ❌ | `Node` supports a walkable parent/child hierarchy (`node.rs`), but no helper implements "climb from the hit leaf mesh to the nearest ancestor carrying selectable metadata." |

## Transform Gizmo

| Feature | Status | Evidence / Gap |
|---|---|---|
| On-screen translate/rotate gizmo (TransformControls equivalent) | ❌ | Repo-wide grep for `gizmo`/`TransformControls` (case-insensitive) — zero Rust hits, no task doc references it either. |
| Per-mode axis constraint (XZ-only translate, Y-only rotate) | ❌ | No gizmo exists to constrain. |
| Keyboard mode switching (G/R/Escape) | ❌ | Same reason. |
| Attach/detach + exclude attached object from idle animation | ❌ | No "currently being manually edited, skip procedural animation" pattern found anywhere in the codebase. |

## Dev Tooling

| Feature | Status | Evidence / Gap |
|---|---|---|
| Central tuning-config object shared by shader defaults, live panel, and render loop | 🟡 | `shader_chunks_params` (`module/shader/shader_chunks_params`) *discovers* tunable params declared via `//@ param:` comments in WGSL — a build/doc-time metadata system, not a runtime object bound to live UI controls. Its own readme states "no real chunk annotated yet" and "no consumers yet." |
| Live DOM dev panel (sliders/selects/color pickers) bound to shader uniforms | ❌ | Grepped `dat.gui\|tweakpane\|imgui\|egui` — zero hits. No runtime GUI crate in the workspace. A task doc (`task/accepting/105_shader_chunks_params_new_crate.md`) explicitly states no windowed/interactive rendering path (no winit/egui) exists anywhere in the workspace today. |
| "Copy settings to clipboard" serializer | ❌ | No dev panel to serialize from. |
| Console scene-state dump (`dumpSceneConfig()`-style) for paste-back editing | ❌ | No dump/serialize-scene function found. Closest *conceptually* related idea: `scene_script`'s rhai "script-as-data" convention (`module/helper/scene_script`) makes the scene definition itself the editable source, so there's nothing to "dump" — a different solution to the same underlying problem (editable defaults). |
| Styled onboarding console message | ⚪ N/A | Trivial, not a meaningful gap either way. |

## Grid & Shader System

| Feature | Status | Evidence / Gap |
|---|---|---|
| Procedural "infinite" grid plane via custom shader | ❌ | Grepped `infinite.*grid\|procedural.*grid` — only hit is the example itself. |
| fwidth-based analytic AA lines | 🟡 | The *technique* is well established elsewhere (`shadowmap`, `hexagonal_map`, `deferred_shading`, `area_light` examples, `line_tools`, `text_msdf`), just never applied to a tiled ground-grid shader. |
| Multi-mode fade curve (linear/smoothstep/exp/exp²) + gamma, shared across effects | ❌ | Grepped `module/shader/*` for `smoothstep\|fade\|gamma` — zero hits. A CPU-side analog exists (`module/helper/animation/src/easing/cubic/bezier.rs` — 24 named easing curves incl. `EaseInExpo`/`EaseOutExpo`), but it's Rust-side value-over-time tweening, not a shader-side, uniform-driven, gamma-reshaped alpha-over-distance function. |
| CPU-built closed boundary polyline (circle + tangent-arc insertion around blockers), uploaded as a fixed-size vec2 uniform array | ❌ | Closest conceptual relative: `module/helper/tiles_tools/src/field_of_view.rs` solves discrete-tile line-of-sight (shadowcasting/raycasting/flood-fill), not continuous-space polyline construction for a shader uniform. |
| GPU nearest-segment-distance test | ❌ | No `distToSegment`/point-to-polyline-distance GLSL/WGSL utility anywhere in `module/shader`. |
| GPU crossing-number point-in-polygon test | ❌ | Grepped `point_in_polygon\|crossing.number\|winding.number` — zero hits in shipped code; only mentions are in two **cancelled** task docs describing work that was never done. |
| Ring/ribbon as two constant-offset copies of one SDF (constant width through bends) | ❌ | Grepped `signed.distance\|SDF` across `module` — zero hits. `line_tools` solves constant-width lines via extruded-quad mesh geometry with joins/caps instead — a different, arguably more natural-fit technique if rebuilt in cgtools. |
| Core/edge color gradient using ribbon falloff as mix factor | ❌ | Trivial GLSL `mix()`, but not evidenced as an existing packaged pattern. |
| Glow effect scoped to a range + clipped to a computed polygon interior | ❌ | Depends on the missing point-in-polygon primitive above. |

## Procedural Geometry

| Feature | Status | Evidence / Gap |
|---|---|---|
| Ship hulls composited from primitives (Box/Cone/Cylinder → named variants) | ❌ | `module/helper/primitive_generation/src/primitive.rs` exposes exactly 4 functions (`curve_to_geometry`, `contours_to_fill_geometry`, `plane_to_geometry`, `path_to_points`) — no cube/cylinder/cone/sphere/platonic-solid generators anywhere. cgtools' actual 3D content pipeline is OBJ/glTF loading (`obj_viewer`, `obj_load`, `gltf_viewer`) — an authored-mesh workflow, not procedural-primitive composition. |
| Engine-glow emissive cones via unlit material | ❌ | Same reason — no procedural primitive compositing. |
| Procedural station (cylinder core, torus ring, radial spokes/modules) | ❌ | Same reason. |
| Procedural `DodecahedronGeometry` | ❌ | Grepped `DodecahedronGeometry\|dodecahedron` — only hit is the example. |
| Per-vertex random uniform scaling (irregular rock shapes) | ❌ | No base platonic solid to deform, and no mesh-vertex-deform utility found. |
| Padding an occlusion radius past a mesh's max procedural bulge | ❌ | N/A without the underlying deformation feature. |
| Per-instance random idle rotation on spawn | ❌ | Trivial to write, but no packaged `spinAsteroids`-equivalent convention found. |

## Fleet / Trajectories / Animation

| Feature | Status | Evidence / Gap |
|---|---|---|
| Fixed spec-data table (position/rotation) baked in as source defaults | 🟡 | `scene_script`'s rhai "script-as-data" convention (`docs/invariant/004_script_as_data_purity.md`) is the closest analog, but it's a different serialization surface (`.rhai` script vs. a plain JS array-of-objects constant). |
| Catmull-Rom spline path following with tangent-based orientation | ❌ | Grepped `CatmullRom\|catmull_rom\|spline` — zero Rust hits. `module/helper/animation` has cubic Bezier/Hermite easing for *value* interpolation between two keyframes, not a multi-point spline-through-a-path with tangent/orientation output. |
| Progress/speed/loop-wrap state with a global playback multiplier | 🟡 | `scene_script::tween_binding` gives `.progress()`/`.duration()`/`.with_repeat()`/`.with_yoyo()` for one animated value, but nothing drives a fleet-wide "speed multiplier applied to N independent entities." |
| Play/Pause/Fast-forward UI-driven playback | ❌ | No DOM UI layer to wire buttons to; the underlying `Tween.pause()/.resume()` primitives exist in `scene_script` but aren't hooked to anything. |
| Trajectory ribbon = spline line + waypoint rings + drop-lines | 🟡 | `module/helper/line_tools` renders thick, anti-aliased, dashed polylines with configurable caps/joins — strictly more capable than three.js's `LineBasicMaterial` used here — but there's no composite "ribbon + waypoint rings + drop-lines" helper; you'd hand-assemble the rest on top of the line primitive. |
| Dashed circle (sensor range ring) | ✅ (as a capability) | `line_tools` supports `DashPattern` variants `V1`–`V4` tracking cumulative arc length — dashing a closed circular polyline is directly supported, just not packaged as a named "sensor ring" helper. |
| Dashed drop-line + ground-intersection crosshair mark | 🟡 | Dashing capability present via `line_tools`; no crosshair/ring-mark helper or "project point onto ground plane" utility found. |
| Toggleable per-layer visibility (trajectory/sensor/grid groups) | ✅ | `renderer/src/webgl/node.rs::Node::visibility_set(&mut self, visibility: bool, only_root: bool)` — directly analogous to toggling `group.visible`. |

## Scene Content / World Composition

| Feature | Status | Evidence / Gap |
|---|---|---|
| Particle starfield (Points, per-particle color, size-attenuated) | 🟡 | `GL::POINTS`/`gl_PointSize` is exercised in several examples (`raycaster`, `space_partition`, `attributes_vao`), so the primitive/shader idiom is proven, but nothing builds a large-count, randomly-scattered, per-vertex-colored, distance-attenuated dust field like this starfield. |
| Scene/camera/renderer/controls/groups encapsulation | ✅ (more elaborate) | `renderer/src/webgl/{scene.rs,node.rs,camera.rs,renderer.rs}` form a real scene-graph + renderer object model — a superset of the JS demo's flat `World` class. |
| Named, independently-toggleable group layering | ✅ | Same `Node.visibility_set` API; hierarchical by nature rather than a flat dictionary of six groups. |
| Material/color palette tuned to match a reference screenshot | ⚪ N/A | Content-authoring decision, not a technical feature. |
| Object intentionally positioned below the "ground" plane | ⚪ N/A | Trivial positioning choice. |
| `userData`-style arbitrary metadata bag on objects | 🟡 | `Node` has `name_set`/`name_get` (`Option<Box<str>>`) but no generic key-value bag like three.js's `object.userData` — anything beyond a name needs a bespoke field or external side-table. |

## HUD / UI (2D DOM overlay)

| Feature | Status | Evidence / Gap |
|---|---|---|
| Full HTML/CSS overlay composited over the WebGL canvas | ❌ | Repo-wide grep for `DOM overlay\|HUD` found nothing relevant; `examples/orrery/webgpu/index.html` was checked directly and is a bare `<canvas>`-only page, confirming this pattern isn't used anywhere in the repo. |
| In-3D-world 2D content via `canvas_renderer` | ✅ (different architecture) | `module/helper/canvas_renderer` renders a 2D scene directly to a WebGL texture for use *inside* the 3D world — the closest cgtools equivalent to "2D over 3D," but it's texture-mapped-onto-geometry, not a DOM/CSS layer composited above the canvas (no pointer-events layering, no CSS styling system). |
| Status bar / mission objectives / tactical-layers panel / unit-info card / toolbar (DOM) | ❌ | Same reason as the overlay item above — no DOM UI convention exists in the Rust/WASM examples. |
| CRT scanline effect | 🟡 (different technique available) | No CSS scanline overlay exists (there's no CSS layer at all), but `renderer/src/webgl/post_processing/` is a real post-processing pipeline (bloom, tonemap, blend) that a scanline *pass* could be added to — a GPU technique swap-in, not a CSS one. |
| CSS design system (scrollbars, glow buttons, backdrop-filter panels) | ⚪ N/A | No CSS in the Rust examples; not a portable "feature" so much as a styling decision that follows once a DOM layer exists. |

## Build Tooling

| Feature | Status | Evidence / Gap |
|---|---|---|
| Vite dev/build/preview tooling, ES modules | ⚪ N/A | `cgtools` builds via `cargo`/`wasm-pack` throughout `examples/minwebgl/*/Cargo.toml` — a different toolchain entirely, not a like-for-like gap. |
| Three.js npm dependency | ⚪ N/A | JS library dependency; not applicable to a Rust workspace. |

---

## Suggested Build Order

Each step below unblocks the next — building out of order means re-doing work once the earlier piece lands.

1. **Scene-graph + render-loop skeleton** on top of `module/helper/renderer` + `mingl::CameraOrbitControls`. Both already exist; this is wiring, not invention.
2. **Object picking / selection.** Either extend `object_picking`'s GPU ID-buffer approach or build real ray-vs-mesh picking. Needed before the gizmo or any unit-info panel means anything.
3. **A dev tuning-panel mechanism** — egui/winit integration, or a hand-rolled DOM-binding pattern like the JS version. This should come *before* the grid shader work, not after: the grid's exact numbers (fade widths, gamma curves, ribbon gaps) were only found by live-tuning sliders in the browser, and there's currently no way to do that at all in cgtools.
4. **The tactical grid shader**, built as a standalone rendering subsystem (like `line_tools`, not a `Material` impl — see the WBOIT finding above) with the fade-curve, boundary-polyline, and point-in-polygon primitives built from scratch. This is the single largest chunk of net-new work in the whole audit.
5. **In parallel once 1–3 land:** the transform gizmo, a Catmull-Rom spline helper on top of the existing Hermite easing (feeding `line_tools` for rendering), and a dodecahedron/platonic-solid primitive generator for the asteroid belt.
