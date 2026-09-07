# OpenPBR Surface Adoption — Plan & Progress

Living plan for adopting the ASWF **OpenPBR Surface** shading model
( <https://academysoftwarefoundation.github.io/OpenPBR/> ) into this crate's
WebGL renderer.

OpenPBR has **no glTF extension of its own**; it ships as **MaterialX** (`.mtlx`)
and **USD**. Content therefore reaches this renderer through two ingestion lanes,
tracked here:

1. **glTF transport** — the ratified Khronos `KHR_materials_*` set (done for
   scalar/color carriers; shading partial).
2. **Native OpenPBR content** — MaterialX `.mtlx` and USD `.usda/.usdc/.usdz`
   (planned in §2), so the actual OpenPBR model libraries can be consumed
   directly rather than via a lossy glTF export.

The two-stage strategy per lane — **load**, then **shade** — is tracked below.

---

## 1. Progress tracking

### 1.1 Done — loader stage

- [x] `OpenPbrParams` value type (`src/webgl/material/pbr.rs`): 15 scalar/color
      carriers, one `Option` per parameter, KHR schema defaults documented per
      field and mapped to the OpenPBR parameter each transports. `is_empty()`
      gates shader selection.
- [x] `loaders::gltf::material_openpbr_params_read` (`src/webgl/loaders/gltf.rs`):
      pure, off-GPU reader for `KHR_materials_ior`, `_sheen`, `_transmission`,
      `_volume`, `_iridescence`, `_emissive_strength`, `_dispersion`,
      `_diffuse_transmission`. Presence semantics: `Some` only while the
      extension is present, already defaulted to the schema default.
- [x] `PbrMaterial::openpbr_params` field wired in `materials_create`.
- [x] Native tests: `tests/gltf_material_extensions_test.rs` (13 cases).
- [x] Docs: crate `readme.md` OpenPBR section + status table; `changelog.md`.

### 1.2 Done — shading stage (opaque path)

- [x] `USE_OPENPBR` define (any carrier present) + `USE_KHR_materials_emissive_strength`
      define (`pbr.rs` `local_defines`).
- [x] Uniforms `ior`, `sheenColorFactor`, `sheenRoughnessFactor`, `emissiveStrength`
      (locations + gated upload with shader-side defaults).
- [x] `main.frag` `USE_OPENPBR` blocks:
      - dielectric Fresnel F0 from IOR `(η−1)²/(η+1)²` (`specular_ior`);
      - energy-preserving per-axis roughness mapping `αt = r²√(2/(1+(1−a)²))`,
        `αb = (1−a)αt` (clamps `r≥0.001`, `a≤0.999`);
      - Smith joint-visibility `V = 0.5/(NoL·Λ(V)+NoV·Λ(L))` with
        `Λ(v)=√(1+((v·T)²αt²+(v·B)²αb²)/(v·N)²)`;
      - fuzz/sheen lobe (`KHR_materials_sheen` → Charlie NDF + Ashikhmin-Premoze
        visibility) for direct + spot + point + IBL;
      - `emissiveStrength` scaling (also honored on the legacy path).
- [x] Fdez-Agüera/three.js multi-scatter on the **IBL term only** (existing
      `integrateBRDF` split-sum `FssEss`/`Fms`) — an approximation, *not* a
      Kulla–Conty LUT; see the pending item in §1.3 / §3.7.

### 1.3 Not implemented / blocked

| Feature | Carrier | Status / blocker |
|---|---|---|
| Refraction (`transmission_weight`) | `KHR_materials_transmission` | **Blocked** — needs a scene-transmission render target + depth (a forward fragment can't see behind the surface). |
| Volume (`transmission_depth`, absorption) | `KHR_materials_volume` | **Blocked** — needs thickness (back-face depth or `thicknessTexture`) + refracted path length for Beer–Lambert. |
| Dispersion (20 / Abbe) | `KHR_materials_dispersion` | **Blocked** — per-channel refraction (3× the transmission pass). |
| Subsurface / translucent scattering | `KHR_materials_diffuse_transmission` | **Blocked** — needs a diffusion pass (per-RGB radius profile). |
| Thin-film iridescence | `KHR_materials_iridescence` | **Not blocked** (pure per-fragment spectral Fresnel) — deferred scope only. |
| Kulla–Conty multi-scatter energy compensation (LUT) | n/a (model fidelity) | **Not blocked** — planned (§3.7); today only the Fdez-Agüera IBL approximation exists, so rough surfaces still lose energy on direct lights. |
| Textured lobe carriers (sheen/transmission/thickness/iridescence/diffuseTransmission color) | all of the above | **Not blocked** (pure plumbing) — deferred scope only. |
| Native OpenPBR content import (MaterialX `.mtlx`, USD `.usda/.usdc/.usdz`) | format lane (§2) | **Not started** — most OpenPBR content ships here, not as glTF; USD/MaterialX runtimes are C++/heavy and not wasm-hostable, so a subset-reader + offline-converter strategy is needed (see §2). |

Blockers are architectural, not mathematical: the renderer is a single-pass
forward shader (`main.frag`) with weighted-blended OIT for transparency; nothing
exists to sample "the scene behind/through a surface".

---

## 2. Native OpenPBR content formats (MaterialX / USD)

OpenPBR is authored and archived as **MaterialX** (`.mtlx`) — the ASWF repo's
`examples/open_pbr_*.mtlx` library (~100 materials) plus the
`reference/open_pbr_surface.mtlx` implementation — and as **USD**
(`.usda`/`.usdc`/`.usdz`; e.g. the Omniverse/OpenUSD OpenPBR material
libraries). glTF carries only a lossy subset through `KHR_materials_*`. "Full
support" therefore adds a second ingestion lane that reads these formats
directly.

### 2.1 Canonical internal surface: `OpenPbrSurface`

Introduce a native parameter surface holding the full OpenPBR set (41 params:
`base_weight/color/diffuse_roughness/metalness`, `specular_*`, `subsurface_*`,
`transmission_*`, `thin_film_*`, `coat_*`, `fuzz_*`, `emission_*`,
`geometry_*`). Both lanes converge on it:

- glTF `KHR_materials_*` carriers → `OpenPbrParams` (existing, presence-based)
  → `OpenPbrSurface`;
- `.mtlx` / `.usda` readers → `OpenPbrSurface` directly.

`OpenPbrSurface` is the single input to the material/shader path (the current
`USE_OPENPBR` uniforms are a subset of it).

### 2.2 Feasibility & blockers

- The **USD** (Pixar) and **MaterialX** (ASWF) C++ runtimes are not hostable in
  wasm. **MaterialX** additionally has **no usable Rust crate** on crates.io
  (`materialx` / `materialx-sys` are v0.0.0 placeholder bindings) — a `.mtlx`
  reader must be purpose-built (the format is XML).
- **USD** has `openusd` (github.com/mxpv/openusd, v0.7): a **pure-Rust, no-C++
  implementation** that reads/writes `.usda`, `.usdc`, and `.usdz`, ships a
  composed `Stage` and opt-in typed schema views including **`UsdShade`**
  (`Material::compute_surface_source` → shader id / nodegraph). Caveats: active
  development with **no API stability below 1.0**, and wasm compatibility is
  **unverified** — needs a compile spike before commitment. (C++ pxr bindings
  such as `usd`/`usd-rs`, `rust-usd`, `pxr_sys` are native-only and rejected for
  the wasm runtime path.)
- Native binary **`.usdc`/`.usdz`** stays feasible only via `openusd` (above); if
  the wasm spike fails, they fall back to offline conversion (§2.3 N4).
- Purpose-built readers for the **OpenPBR subset** mapping onto `OpenPbrSurface`
  remain the core work and are off-GPU + natively testable. USD materials that
  carry OpenPBR surface as an embedded MaterialX nodegraph (`UsdMtlx`) or a
  referenced `.mtlx` asset still resolve to the **same MaterialX parameter
  vocabulary** — so one shared OpenPBR-parameter extraction core serves both the
  standalone `.mtlx` reader (N2) and the USD reader (N3).
- Record an ADR (root `docs/adr/`) for the runtime-vs-offline split before N2.

### 2.3 Phased plan

Status: **N1 complete, N2 landed** — `OpenPbrSurface` (type, spec defaults,
glTF-carrier mapping) in `src/webgl/material/openpbr_surface.rs`; the `.mtlx`
reader in `src/webgl/loaders/openpbr_mtlx.rs`; native tests in
`tests/openpbr_mtlx_test.rs` and `tests/openpbr_gltf_mapping_test.rs`. N3/N4
are still open.

- **N1** — `OpenPbrSurface` type + mapping from the existing glTF carriers;
  keep `OpenPbrParams` presence semantics for glTF, add native defaults for the
  full parameter set.
- **N2** — `.mtlx` reader (OpenPBR subset): resolve material surface instances
  of `open_pbr_surface.mtlx` from their `<input name="…" value="…">` wiring;
  texture inputs deferred to the §3.1 texture plumbing; unresolvable graphs are
  reported, never silently dropped.
- **N3** — USD reader. If the `openusd` wasm spike (compile under
  `wasm32-unknown-unknown`) passes, layer it as an optional
  `native-formats` feature and read `.usda`/`.usdc`/`.usdz`: walk `Material`
  prims → `compute_surface_source` → extract the OpenPBR/MaterialX surface
  inputs via the shared N2 parameter core. On spike failure, fall back to a
  hand-rolled `.usda` text-subset reader, with `.usdz`/`.usdc` offline-only
  (§2.3 N4).
- **N4** — authoring/converter path for real content: export the same material
  to glTF + `KHR_materials_*` for the browser runtime, and keep parameters with
  no KHR carrier in a private `OPENPBR_materials` JSON extension on the glTF
  material (the loader already reads arbitrary extension JSON by name), so
  nothing is lost on round-trip.

### 2.4 Model sources for these formats

- MaterialX: `AcademySoftwareFoundation/OpenPBR` `examples/open_pbr_*.mtlx`
  (native import once N2 lands; offline render in Arnold / MaterialX Viewer as
  ground truth meanwhile).
- USD/OpenPBR: Omniverse OpenPBR material library (OpenUSD + MaterialX);
  Physically Based / MaterialX libraries that publish `.mtlx` or USD variants.
- Cross-format suites: the same material authored as `.mtlx`, `.usda`, and glTF
  `KHR_materials_*` must converge to one `OpenPbrSurface` (equivalence tests,
  §4.1).

---

## 3. Renderer pipeline adjustments (to fully support OpenPBR)

Current frame shape (from `docs/feature/001_pbr_rendering_core.md`):
**opaque → transparent (WBOIT) → resolve/composite → post → display**, allocated
by `src/webgl/renderer.rs`. The new work inserts passes in and around this, in
dependency order:

### 3.1 Textured lobe carriers (unblocked — do first)
Plumbing only. Add `Option<TextureInfo>` slots for
`sheenColorTexture`, `sheenRoughnessTexture`, `transmissionTexture`,
`thicknessTexture`, `iridescenceTexture`, `iridescenceThicknessTexture`,
`diffuseTransmissionTexture`, `diffuseTransmissionColorTexture`; parse them in
the loader (reuse the generic `parse_ext_texture_info`), add sampler uniforms +
UV defines + `configure()` unit assignment + `bind()` unit binds. Texture-unit
headroom: units 0–12 are used; IBL starts at 16 (`ibl_base_texture_unit`), so
13–15 are free.

### 3.2 Thin-film iridescence (unblocked — can land independently)
In-fragment spectral Fresnel (Airy/etalon `Δφ = 2π·2·d·n₂·cosθ₂/λ`, thickness
min/max from `iridescenceThickness{Min,Max}imum`, `iridescenceIor`). Only IBL
wrinkle: the prefiltered env map is monochromatic — tint per channel or 3-tap.

### 3.3 Transmission render target (unblocks refraction, volume, dispersion)
Mirror the existing extra-pass pattern (`post_processing/`, `shadow.rs`, PMREM):

1. Allocate a transmission color (+ reuse depth) target in `renderer.rs`.
2. After the opaque resolve and **before** the transparent pass, make the
   opaque scene image available to transmissive materials as a texture.
3. In the transmissive material path compute the refraction direction via Snell
   with `ior`, apply roughness-driven blur, and sample the transmission texture
   (depth-guided parallax like the three.js `TransmissionPass` approach).
4. Route `transmission_factor` materials into the transparent pass.

### 3.4 Volume thickness + Beer–Lambert (on top of 3.3)
- Thickness source: bake/`thicknessTexture`, or render transmissive back faces'
  depth into a thickness target (leverage existing `doubleSided`/`faceDirection`).
- `T = attenuationColor^(d / attenuationDistance)` along the refracted ray
  (corrected IOR path length), combined with the transmission tint.

### 3.5 Subsurface / diffuse transmission (on top of 3.3)
- Screen-space (or texture-space) diffusion profile over scattered irradiance,
  per-RGB `subsurface_radius`/`radius_scale` (Burley profile). Compose
  `diffuseTransmission` as the diffuse BTDF under the volume surface.

### 3.6 Dispersion (on top of 3.3)
- Per-channel IOR from Abbe (`20/Vd`, default 20), 3 transmission samples
  (R/G/B) — the transmission pass executed per channel, or an analytic
  approximation.

### 3.7 Kulla–Conty multi-scatter energy compensation (LUT)
Single-scatter GGX loses energy on rough surfaces (light bouncing between
microfacets is uncounted); the current code only applies the Fdez-Agüera /
three.js approximation to IBL (§1.2). Implement the spec-sanctioned Kulla–Conty
compensation (OpenPBR spec § Microfacet model cites Kulla 2017):

- **Background**: Kulla & Conty, *Revisiting Physically Based Shading at
  Imageworks*, SIGGRAPH 2017 course
  ( <https://blog.selfshadow.com/publications/s2017-shading-course/> ); real-time
  adaptation: Fdez-Agüera, *A Multiple-Scattering Microfacet Model for Real-Time
  Image Based Lighting*, JCGT 8(1):3
  ( <https://jcgt.org/published/0008/01/03/> ).
- **Data**: directional-albedo LUT `E(μ, α)` (2D) and hemisphere average
  `E_avg(α)` (store as a second channel / row) — precompute once at startup
  exactly like the existing `integrateBRDF` (no closed form for GGX). Both are
  Fresnel-free; color is applied analytically at runtime.
- **Application**: `f_ms = (1−E(μ_o))·(1−E(μ_i)) / (π(1−E_avg))` added to
  `f_single`; per-bounce average-Fresnel weighting for tinted (metal) surfaces.
  Compensates **both** direct punctual lights (sample `E(μ_o)` once, `E(μ_i)`
  per light) and IBL — replacing the IBL-only approximation. New sampler
  uniform + texture unit (headroom 13–15).
- **Independent** of §3.1–3.6 — can land anytime, benefits rough metals /
  dielectrics immediately.

### Sequencing
1. 3.7 Kulla–Conty LUT is independent — land before/along 3.1–3.6 (energy
   correctness is a prerequisite for meaningful visual comparisons).
2. 3.1 textured carriers + 3.2 iridescence (no new passes).
3. 3.3 transmission pass → unblocks 3.4 volume and 3.6 dispersion.
4. 3.5 subsurface (diffusion pass) last.

The §2 native-format workstream (N1–N4) is parallel and off-GPU; its
`OpenPbrSurface` output feeds the same material/shader path these passes consume.

Optional structural note: once the transmission pass lands, graduate the
`USE_OPENPBR`-guarded blocks in `main.frag` into a dedicated `OpenPbrMaterial`
+ shader if the define-count/`main()` complexity warrants it.

---

## 4. Testing plan

### 4.1 Native / unit (no GPU)
- Extend `gltf_material_extensions_test.rs` for texture-carrier parsing once 3.1
  lands (index/texCoord/default semantics).
- Add `.mtlx` / `.usda` parse fixtures (embedded + vendored ASWF `open_pbr_*.mtlx`
  samples) once N2/N3 land, plus **cross-format equivalence**: the same material
  authored as `.mtlx`, `.usda`, and glTF `KHR_materials_*` must yield one
  `OpenPbrSurface`.
- Lift the pure math out of the shader into Rust where cheap (roughness→`αt/αb`
  mapping, Schlick dielectric F0, Beer–Lambert) and native-test it — same
  off-GPU pattern as `attribute_descriptor_make` / `required_extensions_check`.
- Unit-test the Kulla–Conty LUT generator once §3.7 lands: `E(μ, α) ∈ [0, 1]`,
  `E_avg(α)` monotonic in α, and a white-furnace (`∫ BRDF ≤ 1`) converging to
  1 with compensation applied (direct and IBL).

### 4.2 Shader compile (headless browser, CI-only)
- Add `USE_OPENPBR` (and later per-lobe define) compile cases to
  `clearcoat_anisotropy_shader_tests.rs` / `legacy_glsl_shader_compile_test.rs`.
  These run via `cargo test --target wasm32-unknown-unknown` (browser driver) —
  the environment here cannot run them locally, so treat CI as the gate.

### 4.3 Visual / pixel (browser)
- One `examples/minwebgl/*` per lobe (or one combined viewer) rendering the
  sample assets below; verify via the repo's `browsee` pixel-verification flow
  (see prior tasks e.g. 191/197/251).

### 4.4 Reference models & visual comparison

OpenPBR has **no official glTF export**, so compare in tiers; once the §2 native
lane (N2/N3) lands, `.mtlx`/`.usda` are also importable for direct round-trips:

**glTF transport correctness** — Khronos `glTF-Sample-Assets`
( <https://github.com/KhronosGroup/glTF-Sample-Assets> , `Models/`). These are
the canonical `KHR_materials_*` test assets; each folder ships a `screenshot/`
reference image. Purpose-built side-by-side grids exist exactly for
golden/comparison checks:

- `CompareIridescence`, `CompareSheen`, `CompareTransmission`, `CompareVolume`,
  `CompareDispersion`, `CompareAnisotropy`, `CompareEmissiveStrength`,
  `CompareClearcoat`.
- Iridescence: `IridescenceLamp`, `IridescenceSuzanne`, `IridescenceAbalone`,
  `IridescenceMetallicSpheres`, `IridescenceDielectricSpheres`,
  `IridescentDishWithOlives`.
- Sheen/fuzz: `SheenChair`, `SheenCloth`, `SheenTestGrid`, `GlamVelvetSofa`,
  `SheenWoodLeatherSofa`.
- Transmission/volume: `TransmissionTest`, `TransmissionRoughnessTest`,
  `TransmissionOrderTest`, `TransmissionThinwallTestGrid`, `AttenuationTest`,
  `DragonAttenuation`, `GlassVaseFlowers`, `StainedGlassLamp`,
  `SunglassesKhronos`.
- Dispersion: `DispersionTest`, `DragonDispersion`.
- Diffuse transmission: `DiffuseTransmissionTest`, `DiffuseTransmissionPlant`,
  `DiffuseTransmissionTeacup`.
- Anisotropy: `CarbonFibre`, `AnisotropyStrengthTest`, `AnisotropyRotationTest`,
  `AnisotropyBarnLamp`.
- Emissive strength: `EmissiveStrengthTest`.

**OpenPBR ground truth** (the model itself, not the glTF subset) —
`AcademySoftwareFoundation/OpenPBR` repo ships `examples/open_pbr_*.mtlx`
(~100 materials: metals, dielectrics, glass, skin, cloth, velvet) plus the
`reference/open_pbr_surface.mtlx` implementation. Before N2 these render offline
for comparison; after N2 they are native import fixtures too:

- **Ideal offline reference**: Arnold (Autodesk, the OpenPBR reference renderer),
  Adobe Substance 3D, MaterialX Viewer, or NVIDIA Omniverse's OpenPBR material.
- **Ideal real-time reference**: Khronos `glTF-Sample-Viewer`
  ( <https://github.com/KhronosGroup/glTF-Sample-Viewer> ) — the canonical
  implementation of the `KHR_materials_*` extensions — and three.js examples
  (`webgl_materials_transmission`, `webgl_materials_sheen`,
  `webgl_loader_gltf_iridescence`, `webgl_materials_physical*`).
- **USD/MaterialX content libraries**: the Omniverse OpenPBR material library
  and Physically Based `.mtlx` libraries — consumed directly by N2/N3 or through
  the N4 offline converter for the browser path.

Workflow: per lobe, load the glTF-Sample-Assets model in this renderer
side-by-side with the Sample Viewer (real-time parity) and its `screenshot/`
reference; for pure-OpenPBR parity, import/render the matching `.mtlx`/`.usda`
(the renderer once N2/N3 land; otherwise Arnold) and eyeball the same
camera/lighting.

---

## 5. Definition of done (per phase)

- Loader phase: every listed carrier read + native-tested + docs in sync.
- Native-format lane (N1–N4): `OpenPbrSurface` canonical type in place; `.mtlx`
  and text `.usda` subset readers converge fixtures + cross-format equivalence to
  one `OpenPbrSurface`; runtime-vs-offline split recorded in an ADR; `.usdc`/
  `.usdz` and full USD/MaterialX runtimes explicitly out of scope for wasm, with
  the N4 converter path as the documented route for such content.
- Shading (opaque): `USE_OPENPBR` compiles in CI, energy conservation holds
  (white-furnace spot check), no regression for extension-free assets.
- Transmission phase: refractive sample matches Sample Viewer + `screenshot/`
  within tolerance; volume Beer–Lambert verified on `DragonAttenuation`.
- Subsurface phase: diffusion radius matches profile; `DiffuseTransmission*`
  assets match reference.
- Final: the whole OpenPBR carrier set renders on `Compare*` grids within the
  real-time reference's tolerance, with energy + numerical-stability spot checks
  (roughness 0, grazing angles, `NoV→0`/`NoL→0`). §3.7 closes the direct-light
  energy gap: white-furnace total ≈ 1 for rough white surfaces, and rough-metal
  brightness/saturation matches offline Arnold renders of the ASWF metal `.mtlx`
  (gold/copper) rather than reading too dark.
