# decisions — cgtools

Decision log for the cgtools task system.

Distribution: project task backlog — visible to all task contributors.

State legend: 🔍 Unverified · ✔️ Verified · 🚧 Blocked · ✅ Decided · ➖ Cancelled

Format rule: ✅ Decided entries show only the selected option as a single collapsed statement with rationale — rejected alternatives are dropped. ✔️ Verified entries retain the full Options section byte-for-byte plus a Confirmed field citing verification evidence. 🔍 Unverified entries show either the full Options section with a recommendation, or (once an assumption is formulated) a single Assumed statement with a verification mechanism and contingency. 🚧 Blocked entries retain the full Options section plus mandatory Blocked-on/Blocks fields. ➖ Cancelled entries preserve the original analysis plus a mandatory Reason field.

4 entries · 1 cancelled · 3 decided

---

## Index

| ID | Question | State | Owner | Date | Gated by |
|----|----------|-------|-------|------|----------|
| Q-01 | Split, narrow, or plan-extract task 001? | ➖ Cancelled | i4@wbox.pro | 2026_08_08 | — |
| Q-02 | How should `ImageSource::Encoded` decoding be implemented across `tilemap_renderer` backends? | ✅ Decided | user1@w002 | 2026_08_11 | — |
| Q-03 | How should shader-chunk tunable parameters be declared, discovered, and range-resolved? | ✅ Decided | user1@w002 | 2026_08_13 | — |
| Q-04 | Where does `primitive_generation` sit on the L0-L5 rendering-layer ladder? | ✅ Decided | user1@w002 | 2026_08_16 | — |

---

## Q-01 — Task 001 split strategy

**➖ Cancelled · i4@wbox.pro · 2026_08_09**
Should task 001 (SPRAWL) be split into 5 sibling tasks, left as a single broader-scoped task, or extracted into a slim Task plus a Governing Plan with 5 Phases?

Task 001 bundles all 5 Development Milestones (Wasm bridge/canvas; terrain/hydrology/shoreline; hub placement/traffic/roads/bridges; parcel subdivision/labels; AI integration/segmentation/polish) as a single deliverable, which fails the Readiness Verification Gate's D2 (MOST Goal Quality — Scoped) dimension (`tsk.rulebook.md § Task File : Readiness Verification Gate`).

**A: Split into 5 sibling tasks**
One per Development Milestone, linked via `related_tasks`/`blocked_by`. Each milestone already has concrete deliverable bullets in the source spec; achievable without fabricating detail.

**B: Leave as one task, narrow scope**
Accept a single task but shrink Out of Scope until only one milestone's worth of deliverable remains; defer the rest to follow-up tasks filed later.

**C: Extract to a Governing Plan**
Slim Task plus a `pln.rulebook.md`-compliant Plan with 5 Phases (the existing `### Phase 1`–`### Phase 5` headers in the Technical Specification already match Plan's own Phase vocabulary). Requires authoring per-phase Estimated-Time/Outputs/Steps detail (`pln.rulebook.md` TP037) not present in the source material today — needs real estimation input from the task owner, not invention.

→ **Recommended: A** — leverages `tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate` (adapted from crate-boundary to milestone-boundary partitioning, since no D2-specific split procedure exists); B just relabels 4/5 of the scope as undefined future work rather than resolving it; C is blocked on per-phase estimation data that doesn't exist yet without inventing it.

**Assumed A** based on the milestone boundaries already present in task 001's own Development Milestones section. Verification mechanism: each resulting task's own Readiness Verification Gate — if the milestone boundaries were wrong, at least one resulting task would fail D1/D2/D6/D7/D8. If wrong: fold surviving milestones back toward Option B, or revisit Option C once real per-phase estimation data exists.

**Confirmed** by tasks 002-006's own `## Verification Record` sections (durable, `task/verified/00{2..6}_*.md`) — all 8 Readiness Verification Gate dimensions 🟢 on both passes for all 5 tasks (40/40), including one genuine adversarial catch-and-fix (002's D6, an overbroad dependency-registration bullet). Reconfirmed at the option-selection level (A vs B vs C) by the Gate Check below (2026-08-08, this session).

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 1/1

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| G1 | Option A selection validity (vs B, C) | — | 🟢 | Adversarial checks against A: (1) B merely relabels 4/5 of scope as undefined deferred work rather than resolving it — weaker than A; (2) C is genuinely blocked on real per-phase estimates that don't exist without inventing them, not just deprioritized; (3) milestone boundaries hold empirically — 40/40 dimension passes across 002-006, including D8 Crate Single Responsibility on each; (4) the linear `blocked_by` chain matches the original spec's own data dependencies (M3 needs M2's terrain grid; M4 needs M3's road network), not an arbitrary serialization; (5) sunk-cost bias risk (A was already executed before this check) is partially mitigated by mechanical evidence (40/40 passes, one real D6 catch) rather than narrative self-assurance — full mitigation is deferred to the human ENDORSE step, which this gate does not substitute for | — |
| **Total** | | — | 🟢 | 0 open | — |

**Aggregate verdict:** PASS — Option A holds under adversarial challenge; no rejected finding. State → ✔️ Verified (superseded below).

**Reason (➖ Cancelled, 2026-08-09):** Before the ENDORSE step above ever fired, the filer (i4@wbox.pro) cancelled the entire SPRAWL initiative outright — task 001 and its milestone split 002-006 alike — as exploratory/idea-stage work, not committed for implementation. This moots the question this decision resolved: Option A (split into 5) was the correct decomposition *if* SPRAWL were to proceed, and remains validly confirmed as that (40/40 gate passes stand unchanged) — but with no implementation proceeding at all, no split/narrow/plan-extract strategy is needed. All 6 tasks (001-006) moved to `task/cancelled/`; Tasks Index reindexed accordingly.

---

## Q-02 — `ImageSource::Encoded` decoding strategy across backends

**✅ Decided · user1@w002 · 2026_08_11**
How should `tilemap_renderer`'s `ImageSource::Encoded` (PNG/JPEG/etc. bytes) be decoded across backend families, closing the gap `roadmap.md`'s "webgl adapter gaps" section documents?

**Reject bundling the full `image` crate into `adapter-webgl` (the assumed Option 1 from the prior unverified analysis); decode by backend family instead: web/wasm backends (`adapter-webgl`) decode via browser-native mechanisms — `minwebgl::create_blob` (bytes → Blob → object URL) feeding the existing `HtmlImageElement`-based async upload path already used for `ImageSource::Path`, reusing the already-proven Blob-upload pattern in `renderer/src/webgl/loaders/gltf.rs` — rather than reimplementing browser decoding in Rust; native (non-wasm) backends (`adapter-svg`) use a minimal dependency on a single image format's decoder (the `png` crate — the codebase's own current-evidence default; revisable if real production art pipelines need a different format) rather than the full multi-format `image` crate.** This also resolves a pre-existing over-dependency: `adapter-svg`'s `dep:image` pulled in every format codec `image` supports plus `rayon`, for a usage surface (header-only dimension parsing + PNG-only encoding) that never performed a full multi-format pixel decode. Non-PNG `Encoded` bytes on the native/`adapter-svg` path degrade gracefully after this change (dimension extraction returns `None`, same as today's malformed-bytes case) rather than the multi-format decoding the prior `image`-crate-backed `image_dimensions` supported — MIME-type detection (a separate, byte-pattern-only function) is unaffected and still recognizes PNG/JPEG/GIF/WebP/SVG regardless. Implemented by `task/completed/092_tilemap_renderer_webgl_encoded_image_decode.md` (web half) and `task/completed/093_tilemap_renderer_svg_minimal_png_decoder.md` (native half).

---

## Q-03 — Shader-chunk tunable-parameter declaration, discovery, and range-resolution strategy

**✅ Decided · user1@w002 · 2026_08_13**
How should chunk-level tunable parameters (function argument, compile-time define directive, uniform, attribute, texture — per user request) be declared and discovered by a new crate, and how should a parameter with no declared range get one?

**Add a new repeatable `//@ param: <name> <kind> <type> [range(min, max)]` line to the existing flat manifest header block** (`shader_chunks_core`'s established `//@ name:`/`//@ description:`/`//@ tags:`/`//@ depends_on:`/`//@ export:` convention, confirmed by reading `shader/fbm3/fbm3.wgsl` and `shader/hash21/hash21.wgsl` directly — one comment block at the top of the file, `export` already proving the repeatable-line pattern) — rather than scattering inline annotations next to each individual declaration in the WGSL body, which would break the file's single-header-block shape and couple manifest position to body position. `kind` is one of the 5 requested literal spellings (`argument`, `define`, `uniform`, `attribute`, `texture`); `type` is the WGSL type token copied verbatim from the adjacent real declaration.

**Reject pure syntactic inference** (scanning function signatures/bodies for anything that looks like a parameter) as the sole discovery mechanism: it cannot distinguish a genuine tunable knob (an octave count) from a structurally-required non-tunable argument (a sample coordinate) — chunks must explicitly opt a value into tunability via `//@ param:`, mirroring how `export` already requires explicit opt-in rather than exporting every function found.

**Range resolution:** a declared `range(min, max)` always wins. When absent, a deterministic (no randomness) two-stage heuristic infers one — name-substring pattern match first (`octaves`/`count`/`steps`/`iterations` → `[1, 8]`; `seed` → `[0, 65535]`; `angle`/`rotation` → `[0.0, τ]`; `scale`/`frequency`/`freq` → `[0.1, 10.0]`; `amplitude`/`weight`/`opacity`/`alpha`/`mix`/`blend` → `[0.0, 1.0]`; `radius`/`size`/`width`/`height` → `[0.0, 100.0]`), falling back to a WGSL-type-keyed default (`bool`/`texture_*` → no numeric range; `u32`/vec-of-`u32` → `[0, 16]`; `i32`/vec-of-`i32` → `[-16, 16]`; `f32`/vec-of-`f32` → `[0.0, 1.0]`, matching this codebase's own noise chunks' unit-interval convention) when no name pattern matches. Every inferred range is tagged `RangeSource::Inferred` versus a declared range's `RangeSource::Declared`, so a consumer can always tell which happened.

**Scope boundary:** discovery (`shader_chunks_params`, a new crate) and its CLI surface (`shader_chunks`'s `tunables` command) are the full extent of this decision — annotating the 4 existing bundled chunks with real `//@ param:` lines, and any live GPU-backed parameter-preview UI, are both explicitly out of scope (YAGNI — no `//@ param:` line exists anywhere yet to annotate against, and no windowed/interactive rendering path exists anywhere in this workspace to preview into; the mechanism is independently valuable and testable via self-contained fixtures without either).

---

## Q-04 — `primitive_generation`'s L0-L5 ladder placement

**✅ Decided · user1@w002 · 2026_08_16**
Where does `primitive_generation` sit on `rulebook.md`'s L0-L5 rendering-layer ladder, given its `future`/`math`/`diagnostics` minwebgl feature gate superficially resembles `animation`'s beside-the-ladder shape, but `docs/layer/001_l0_drivers.md` already flags a contradiction: direct `WebGl2RenderingContext` imports in `src/primitive_data.rs`?

**Classify at L4 (scene model), as a second occupant alongside `renderer`'s glTF loaders — not beside-the-ladder like `animation`.** Full-file reads of all 3 source layers (`lib.rs`, `primitive.rs` — 458 lines, `primitive_data.rs` — 302 lines) plus `Cargo.toml` show the crate is not structurally comparable to `animation`: `animation`'s `minwebgl`/`mingl` dependencies are genuinely math-only (`grep -rn "WebGl2RenderingContext\|WebGlBuffer\|create_buffer\|renderer::" module/helper/animation/src/` → zero matches), but `primitive_generation` additionally depends on `dep:renderer` (`features = ["full"]`, the full L3 rendering engine) and its `primitive_data.rs` module — unconditionally compiled, no `#[cfg(feature = ...)]` gate anywhere in the file — imports `WebGl2RenderingContext` plus `renderer::webgl::{AttributeInfo, Geometry, IndexInfo, Material, Mesh, Node, Object3D, Primitive, Scene, loaders::gltf::GLTF, material::PbrMaterial}`: real L3 rendering-engine domain types, not math utilities. `primitives_data_to_gltf` (`primitive_data.rs:141-289`, publicly re-exported via `mod_interface!`) takes a live `gl: &WebGl2RenderingContext`, calls `gl.create_buffer()` / `gl::buffer::upload` / `gl::index::upload` directly, and returns a `renderer::webgl::loaders::gltf::GLTF` — the exact struct type `rulebook.md`'s L4 row already cites as "glTF via `renderer` loaders." Requiring a live GL context does not disqualify L4 membership: `renderer`'s own loader functions (`module/helper/renderer/src/webgl/loaders/gltf.rs`, e.g. lines 518/610/657/744/867/965/1042/1262) likewise take `gl: &gl::WebGl2RenderingContext` throughout — that row's existing occupant has the identical shape. `primitive_generation` is therefore a second L4 occupant: it consumes L3 (`renderer`) and produces L4-shaped (`GLTF`) output, via procedural parameters instead of parsed file bytes — an alternative glTF *producer*, not a horizontal math capability.

**Reject beside-the-ladder classification** (the shape `animation` uses): the GL/renderer coupling here is load-bearing, not incidental — `dep:renderer`/`dep:minwebgl` are pulled in by the crate's `default = ["enabled"]` feature with no feature-gated path to compile `primitive_generation` without them, so splitting the GL-coupled half into its own crate would be a real refactor, not a documentation nuance; that refactor is out of this decision's scope, to be filed as a follow-up task only if a concrete future need justifies it. The pure-geometry-generation code in `primitive.rs` (458 lines, zero GL/renderer imports, confirmed via the same grep) exists to feed `primitives_data_to_gltf`'s `PrimitiveData` input, not to stand alone as the crate's primary value proposition the way `animation`'s pure-math API does — so the crate as a whole (one Cargo unit, one placement) follows its load-bearing half, not its larger-by-line-count half.

---
