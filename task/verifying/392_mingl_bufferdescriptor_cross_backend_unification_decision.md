# Decide fate of `mingl::BufferDescriptor` — cross-backend unification or drop

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **started_at:** 2026-08-19 05:09:12
- **expires_at:** 2026-08-19 07:09:12
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/mingl
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **unverified_at:** 2026-08-19 05:09:04
- **unverified_by:** unknown
- **in_motion:** true
- **verifying_at:** 2026-08-19 05:09:12
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/

## Goal

`module/min/mingl/src/buffer.rs` is fully dead code: a commented-out `BufferDescriptor` struct
(builder pattern: `.vector()/.offset()/.stride()/.divisor()`, `new<I: IntoVectorDataType>()`), gated
out via `mod_interface!`'s commented `orphan use { // BufferDescriptor, };`.

This session's investigation found the capability it targets — describing one vertex attribute's GPU
buffer layout via a chained builder — has been **hand-built independently in every backend that
needs it**, not solved once:

- `module/min/minwebgl/src/buffer.rs` — live `BufferDescriptor`, identical shape to the dead mingl
  draft (`vector`/`offset`/`stride`/`divisor`), real consumers across `line_tools`, `renderer`,
  `primitive_generation`, `gpu_hal`, and 15+ examples.
- `module/min/minwebgpu/src/layout/vertex_attribute.rs` — live `VertexAttribute`, same
  builder-pattern shape (`new().location().format()`), but a **different field set**: WebGPU's model
  is `shaderLocation`+`format` (a GPU-specific enum)+`offset`, not WebGL's `vector-type`+`offset`+
  `stride`+`divisor`. Plus `module/min/minwebgpu/src/descriptor/buffer.rs`,
  `layout/vertex_buffer.rs`, `state/vertex.rs` — a parallel cluster of buffer/layout description code
  with no shared root.

So the dead mingl file isn't "solved elsewhere, safe to delete" in the simple sense (contrast task
393's `AsBytes` incident, same audit) — the underlying need has been reinvented twice, which is a
real duplication signal. But the two live implementations' field sets differ enough (driven by each
backend's actual GPU API shape) that reviving the dead file verbatim would not serve minwebgpu at
all — it's shaped specifically for WebGL's `vertexAttribPointer` model. A genuine shared abstraction
would need new design work (what's the right common shape across two structurally different GPU
binding models?), not a file revival.

**Git-history investigation (per user request):** searched this repo's full history for
`buffer.rs`'s rollback pattern — `master` plus all 34 locally-fetched branches (including a repo that
has genuine revert-branches as a working pattern, e.g. `revert-12-mapgen-tiles-rendering`), back to
the earliest reachable commit (`dc8c8c1f "initial commit"`, 2024-10-28 — a single squashed commit
importing the whole codebase at once). Result: **`buffer.rs` has exactly 2 commits in this repo's
entire history** — the initial squash (file already dead/commented at that point) and one unrelated
recent commit. No branch, anywhere, shows a different version of this file. If it really was
implemented and rolled back multiple times, that happened before this repo's history begins and
isn't recoverable here — confirming the actual reason needs the original developer's own
recollection or external pre-2024 records, not something derivable from this repo alone.

**This is explicitly a tracking placeholder, not active work.** No implementation should begin
speculatively.

**Related Tasks:** `394` (same shape of open decision, same audit, same user directive to leave open
for developer input). `391`/`393`/`395` (this audit's other 3 incidents, all resolved to concrete
action rather than left open).

## Open Question (needs developer answer)

1. Is a shared `mingl`-level abstraction over WebGL's and WebGPU's vertex-attribute-description worth
   designing, given the two backends' field sets genuinely differ at the GPU-API level? Or is "each
   backend hand-rolls its own descriptor" the correct long-term shape here?
2. If a shared abstraction is wanted: what's the actual common denominator worth extracting (e.g.
   just `offset`, with everything else left backend-specific), and is that worth the design/migration
   cost against two already-working, already-adopted implementations?
3. Does anyone recall why this was apparently attempted and abandoned in mingl specifically
   (pre-dating this repo's history) — was it the same field-mismatch problem found here, or something
   else?

Until answered: `module/min/mingl/src/buffer.rs` stays as dead/commented code; no further action.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 05:09:04 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-19 05:09:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ | CLAIM_VERIFY | verification claimed |

## History

- **[2026-08-19]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) following a user-requested
  dead-code maturity review of `module/math`/`module/min`. User directive: this incident stays an
  open decision for the developer, not a unilateral delete-or-implement call; investigate the current
  alternative implementations and the historical-rollback question first (done, see Goal) — the
  rollback investigation found no evidence in this repo's reachable history, documented above rather
  than guessed at.

- **[2026-08-19]** `ANALYZED` — User requested a deeper pass: which live mechanisms substitute for
  this file, why it was rolled back, and every call site that would benefit. Analysis-only, no
  implementation began, per this task's own "not active work" clause.

  **Mechanisms live today (4, not 2):** beyond the 2 already named in Goal
  (`minwebgl::BufferDescriptor`, `minwebgpu::VertexAttribute`), two more independent reinventions of
  "describe a GPU vertex attribute" exist: `renderer::webgl::geometry::AttributeInfo` +
  `buffer_attribute_info_make()` (`module/helper/renderer/src/webgl/geometry.rs:9`,
  `module/helper/primitive_generation/src/primitive_data.rs:122`) — a WebGL-only mid-level wrapper
  bundling `slot`+`buffer` handle+`descriptor`+`bounding_box`+`.upload()`, and, most importantly,
  **`gpu_hal::types::VertexAttribute`/`VertexBufferLayout`** (`module/helper/gpu_hal/src/types.rs:277`)
  — a `location`+`format`(closed 3-variant `VertexFormat` enum)+`offset` struct that is **already
  genuinely cross-backend**: consumed directly by the WebGL adapter (`gpu_hal/src/webgl.rs:188`),
  the Vulkan/native adapter (`gpu_hal/src/vulkan.rs:1375` via `wgpu::VertexAttribute`), and the
  WebGPU adapter (`gpu_hal/src/device.rs:1968-1980`, converting to `web_sys::GpuVertexBufferLayout`).

  **This directly answers Open Question 1.** A shared cross-backend vertex-attribute-description
  abstraction was already designed and shipped — just one layer up, in `gpu_hal`, not in `mingl`.
  `gpu_hal`'s own answer to the "common denominator" question (Open Question 2) is
  `location`+`format`+`offset`, with stride pulled up to the per-buffer `VertexBufferLayout` level —
  dropping WebGL's `divisor` entirely (not yet needed by any `gpu_hal` consumer). Reviving
  `mingl::BufferDescriptor` as a *second*, `mingl`-level cross-backend abstraction would compete with,
  not complement, the one the project already committed to at the `gpu_hal` layer.

  **Rollback investigation — one new fact.** Beyond the git-history investigation already recorded
  above (2 total commits, no version of this file was ever anything but dead-on-arrival in this
  repo), current `HEAD` (`709f1c1c`, this session's concurrent-actor commit, subject line "feat:
  expand test coverage and document identified bugs") **is itself the file's real, committed
  deletion** — confirmed via `git show HEAD:module/min/mingl/src/buffer.rs` → `fatal: path ...
  exists on disk, but not in 'HEAD'`. Yet the file is back on disk right now as an **untracked** file
  (`git status --porcelain` → `?? module/min/mingl/src/buffer.rs`), byte-identical to the pre-deletion
  content. The deletion was incidental collateral in an unrelated bulk commit (its own message never
  names `mingl`/`buffer.rs`), not a deliberate decision on this file — and even that deletion hasn't
  stuck, consistent with this repo's known concurrent-actor/session-overlap hazard rather than any
  intentional repeated rollback. No evidence anywhere in this repo supports "rolled back several
  times" as a literal implement-then-revert cycle; if that happened, it predates `dc8c8c1f` (the
  2024-10-28 squashed initial commit) and isn't recoverable here.

  **Call sites relevant to this decision** (crates independently maintaining attribute/buffer-layout
  description logic — the duplication this task's Goal already flags, now enumerated):

  | Mechanism | Location | Scope | Real construction call sites |
  |---|---|---|---|
  | `minwebgl::BufferDescriptor` | `module/min/minwebgl/src/buffer.rs` | WebGL only | 110, across ~30 files (see task 394 for the full breakdown) |
  | `minwebgpu::VertexAttribute` | `module/min/minwebgpu/src/layout/vertex_attribute.rs` | WebGPU only | 3 files (`gpu_hal/src/device.rs`, `examples/minwebgpu/deffered_rendering/src/{model,light}.rs`) — WebGPU adoption is far thinner than WebGL's |
  | `renderer::AttributeInfo` | `module/helper/renderer/src/webgl/geometry.rs:9` | WebGL only | 1 direct definition site + 1 builder consumer (`primitive_generation/src/primitive_data.rs:122`) + all `GBuffer`-attachment call sites (`renderer/src/webgl/post_processing/gbuffer.rs`) |
  | `gpu_hal::VertexAttribute`/`VertexBufferLayout` | `module/helper/gpu_hal/src/types.rs:277,289` | WebGL + WebGPU + Vulkan/native | `gpu_hal/src/{webgl,vulkan,device,resource}.rs` — the actual cross-backend answer |

  Still unresolved (unchanged, needs the developer, not this analysis): whether `gpu_hal`'s existing
  abstraction should simply be adopted as the answer and this dead file deleted (matching task 393's
  precedent), or whether a `mingl`-level abstraction is wanted for a different reason (e.g. so
  non-`gpu_hal` consumers of `minwebgl`/`minwebgpu` directly, like `line_tools`/`renderer`'s WebGL
  path, gain a shared type too) — `gpu_hal` is opt-in per `Cargo.toml` feature, not a mandatory
  dependency of either backend crate, so it doesn't automatically reach every existing call site.

- **[2026-08-19]** `EXECUTED` — User directive: apply the same treatment as task 394 (see that task's
  own `EXECUTED` entry) — "full migration" scope. Revived `module/min/mingl/src/buffer.rs` for real,
  wired it into `gpu_hal`, and migrated 3 real call sites off their old per-backend builders.
  `line_tools`'s multi-buffer sites were explicitly deferred (out of scope — see task 394's own
  deferral of the same crate for the parallel reason).

  **Design, revived close to the original shape, not verbatim.** The dead draft's single
  `BufferDescriptor` struct (`vector`/`offset`/`stride`/`divisor` all on one flat type) is replaced by
  a two-level split matching `gpu_hal::VertexAttribute`/`VertexBufferLayout`'s own precedent (see this
  task's `ANALYZED` entry): `VertexAttribute { location, vector : VectorDataType, offset }` (per
  attribute) + `VertexBufferLayout { stride, step_mode : StepMode, divisor : usize, attributes :
  Vec<VertexAttribute> }` (per buffer). `StepMode` (`Vertex`/`Instance`) is the WebGPU/Vulkan-portable
  binary switch (`GPUVertexStepMode`/`VkVertexInputRate`); `divisor : usize` stays a separate,
  explicitly WebGL-only field (`vertexAttribDivisor`'s arbitrary divisor has no WebGPU/Vulkan
  equivalent) rather than folded into `StepMode` — keeps the type honestly meaningful on every
  backend instead of pretending divisor is portable. This answers this task's own Open Question 1/2:
  the right common denominator is exactly `gpu_hal`'s existing `location`+`format`+`offset` split,
  pulled down to `mingl` with `divisor` added back as a clearly-marked WebGL-only extension.

  **`gpu_hal` wiring.** `gpu_hal::types::VertexAttribute`/`VertexBufferLayout` were missing
  `step_mode` support entirely (silently always vertex-rate) — added, with a ripple-fix across all 4
  existing construction sites in `renderer/src/webgpu/geometry.rs` (each now carries
  `step_mode : StepMode::Vertex` explicitly, preserving existing behavior).

  **3 real call sites migrated:**
  - `examples/minwebgl/attributes_vao/src/main.rs` — the AoS case, via the `Attribute` trait; see
    task 394's own `EXECUTED` entry for the trait-level detail. Confirms this task's dead-code
    revival is genuinely consumable end-to-end, not just wired and unused.
  - `module/helper/renderer/src/webgl/post_processing/gbuffer.rs` — the SoA case (4 independent
    single-attribute `GBufferAttachment` variants, one buffer per attribute, not one interleaved
    struct). `attribute_info()`'s match arms now build `(mingl::VertexAttribute, bool)` pairs
    (attribute + a WebGL-only `normalized` flag `mingl::VertexAttribute` doesn't model), bridged to
    `gl::BufferDescriptor::from_vector(...).offset(...).stride(0).normalized(...)` at point of use.
  - `module/helper/primitive_generation/src/primitive_data.rs` — also SoA.
    `buffer_attribute_info_make` simplified from 7 params (`buffer, descriptor : gl::BufferDescriptor,
    offset, stride, slot, normalized, vector`) to 4 (`buffer, attribute : mingl::VertexAttribute,
    stride, normalized`). Its one external caller outside this crate,
    `examples/minwebgl/animation_surface_rendering/src/primitive_data.rs`, updated to match at its
    `geometry_create` call site — found via a workspace-wide grep before declaring the migration
    complete, distinguished from 2 unrelated same-named local functions in other example crates
    (`narrow_outline`, `text_rendering`) correctly left untouched.

  **Why gbuffer.rs/primitive_data.rs got `mingl::VertexAttribute` directly, not the `Attribute`
  trait**, prompted directly by the user's own mid-session question ("can descriptor describe both
  array of structures and structures of arrays as well?"): the 392-level types
  (`VertexAttribute`/`VertexBufferLayout`) are generic over both shapes by construction — AoS is one
  `VertexBufferLayout` with N attributes at different offsets sharing one buffer, SoA is one
  layout/attribute per buffer with `stride` equal to that attribute's own size. The 394-level
  `Attribute` trait is AoS-only by its `T : Pod` bound (one interleaved Rust struct per buffer
  element, by definition) — forcing gbuffer.rs/primitive_data.rs onto it would need artificial
  single-field newtype wrappers per attachment, rejected as over-engineering. See task 394's own
  `EXECUTED` entry, which corrects its "Clean fit" table accordingly for these 2 files.

  **Verification — full native + wasm32 nextest/doctest/clippy, all green**, after 2
  verification-methodology fixes (not source defects): `--all-features` was blindly enabling
  `gpu_hal`'s native-only `vulkan` feature (pulls in `ash`, zero wasm32 support) and
  `gpu_hal_triangle_browser`'s mutually exclusive `webgpu`/`webgl` features (each gates its own
  `fn main()`, together `E0428: main redefined`) under a wasm32 target neither is meant to be
  force-combined for — fixed by scoping each crate's wasm32 clippy invocation to its actual
  target-relevant features instead of a blanket `--all-features`.

  **2 out-of-scope opportunistic fixes**, needed only to unblock the combined final-verification gate,
  not part of this task's own scope — flagged explicitly rather than folded in silently:
  - `module/min/mingl/src/buffer.rs` — `clippy::wildcard_imports` on my own new code
    (`use crate::*;` → `use crate::{ VectorDataType, mem };`).
  - `module/helper/tilemap_renderer/tests/webgpu_backend_test.rs` — pre-existing `clippy::float_cmp`
    debt, confirmed via `git status --porcelain` as untouched by me otherwise; fixed with a scoped
    `#[allow(clippy::float_cmp, reason = "...")]` since every compared value is a hardcoded literal
    with no arithmetic in between (no representation-error risk, exact comparison is the actual test
    intent) — matches this project's own pre-existing `#[expect(..., reason = "...")]` idiom.

  Tier 2 (Dual-Role Self-Check, standing project cap — see `feedback_maav_tier_cap` memory).
  Confirming pass: all 3 call sites compile, migrate off the old per-backend builders as intended, and
  the combined native+wasm32 nextest/doctest/clippy run is green. Adversarial pass: specifically
  hunted for (a) any other external caller of the changed `buffer_attribute_info_make` signature
  missed by the workspace grep — none found beyond the one already fixed; (b) any behavior change
  hidden in the `gbuffer.rs` bridge (stride/offset/normalized values per attachment) — diffed each
  match arm's old literal values against the new `mingl::VertexAttribute` construction field-by-field,
  confirmed identical; (c) whether `line_tools`'s deferral was scope creep avoidance or an
  undocumented gap — confirmed explicitly out of scope per the original "Full migration" plan
  approval, not silently dropped.

- **[2026-08-19]** `VERIFIED` — user directly asked whether the changed examples were manually
  tested; they had not been (the verification above was compile/lint-only, native + wasm32). Closed
  the gap: ran all 3 migrated call sites live in a browser (`longrun`-detached `trunk serve`,
  `browsee` for screenshot/console/pixel-verdict inspection — launched from outside each crate
  directory per the known Trunk+Longrun rebuild-loop hazard).
  - `examples/minwebgl/attributes_vao` — 10 correctly colored/sized/positioned squares (2 VAOs × 5
    points each), matching the crate's own stated purpose.
  - `examples/minwebgl/renderer_with_outlines` (real consumer of `gbuffer.rs`'s `attribute_info()`)
    — loads a 166-mesh/27-texture glTF car model through the full GBuffer deferred-shading +
    outline post-process pipeline; renders correctly shaded and textured. An initial all-black frame
    was software-GL shader-compile/asset-decode latency, not a defect — confirmed via console log
    showing normal glTF-load progress with no errors, then a correct render once compiled.
  - `examples/minwebgl/animation_surface_rendering` (real consumer of the changed
    `buffer_attribute_info_make`, via its own `primitive_data.rs`) — renders a correctly shaped,
    properly textured/shaded Earth sphere from custom-generated primitive geometry.

  No visual defects found in any of the 3. This does not extend to the ~25 files task 394's own fit
  table marks "Unassessed" — those were never in this migration's scope to begin with (see that
  task's own table), not silently skipped during testing.

- **[2026-08-19]** `EXPANDED` — Separate later session picked up the ~25-file "Unassessed" long tail
  this task's own `EXECUTED`/`VERIFIED` entries explicitly deferred. Confirmed via
  `git show --stat fa4041ef -- examples/ module/` (the concurrent-actor commit that swept this work
  in, see below): `area_light/{main,plane}.rs`, `attributes_instanced/main.rs`,
  `attributes_matrix/main.rs`, `deferred_shading/geometry.rs`, `diamond/main.rs`,
  `hexagonal_grid/main.rs`, `make_cube_map/main.rs`, `minimize_wasm/main.rs`, `narrow_outline/main.rs`,
  `obj_load/main.rs`, `obj_viewer/mesh.rs`, `object_picking/main.rs`, `raycaster/main.rs`,
  `space_partition/main.rs`, `text_msdf/{main,text}.rs`, `text_rendering/main.rs`, `wfc/main.rs`, plus
  `module/helper/renderer/tests/geometry_tests.rs` — 20 source files (+ matching `Cargo.toml`
  `mingl`-dependency additions), all onto this task's `mingl::VertexAttribute`/`BufferDescriptor`
  pair (per-attribute/SoA pattern) except `area_light/plane.rs`, which adopted task 394's own
  `Attribute` trait instead (see that task's own `EXPANDED` entry).

  **Independently re-verified, not rubber-stamped**, on 4 of these files directly (byte-level GL
  semantic diff against pre-migration source, matching this task's own `deferred_shading`
  cross-check method above): `deferred_shading/geometry.rs`, `text_rendering/main.rs`,
  `narrow_outline/main.rs`, `area_light/{main,plane}.rs` — all confirmed offset/stride/divisor
  byte-for-byte identical to pre-migration behavior. The remaining files in the list were confirmed
  via live browser rendering (screenshot/console/pixel-verdict per crate) plus the full-workspace
  test/clippy run below, not per-file diff tracing.

  **2 real regressions found and fixed** (both downstream consequences of the migration, not
  semantic-equivalence failures): (1) `area_light/tests/plane_texcoord_test.rs` (a BUG-321 regression
  test) hand-parses `plane.rs`'s `plane_vertices` array via `include_str!` + `.split(',')` — broke
  when the array literal changed from flat `&[f32]` to `&[Vertex]` struct literals; fixed by
  rescoping the split predicate to digit/`.`/`-` characters, format-agnostic either way. (2)
  `text_rendering`/`narrow_outline` both had a `buffer_attribute_info_make` helper whose only `Err`
  path was a dead match arm removed by the migration, triggering hard-denied
  `clippy::unnecessary_wraps`; fixed by returning bare `AttributeInfo` instead of
  `Result<AttributeInfo, WebglError>` in both (6 call sites' `.unwrap()` removed combined), plus a
  stale doc comment on `narrow_outline`'s copy still describing the old fallible signature.

  **Full verification: `verb/test` genuinely green** — native nextest/doctest/clippy, wasm32
  compile-check across all browser examples, wasm32 test execution (including
  `vertex_attribute_tests`) — exit 0, 1372s, via `longrun`.

  **Concurrent-actor commit discovered mid-verification**: another session under the same git
  identity committed the working tree (`fa4041ef`, "feat: add GUI support and enhance example
  tests") while the final `verb/test` run was still executing. Verified via `git show fa4041ef` that
  both fixes above landed intact — nothing lost. `fa4041ef` also includes unrelated GUI-support work
  (`simple_pbr`'s `gui.js`/`lil_gui.rs`/etc.) — not part of this migration, not attributed to it here.

  Tier 2 (Dual-Role Self-Check, standing project cap). Confirming pass: 4 files directly
  diff-verified byte-equivalent, 2 real regressions found and fixed, full native+wasm32 suite green.
  Adversarial pass: specifically hunted for (a) any other file sharing the `unnecessary_wraps`
  pattern beyond the 2 found — full-workspace `cargo clippy --keep-going` swept clean, none found;
  (b) whether the concurrent commit dropped or altered either fix — `git show fa4041ef:<path>`
  confirmed byte-identical to what was applied; (c) whether `area_light`'s visual "jagged sawtooth"
  symptom (flagged during browser testing) traces to this migration — diff-confirmed both
  `main.rs`/`plane.rs` semantically unchanged, so the symptom (if real) has a cause elsewhere,
  out of scope for this migration task.

  **`tsk .verify_pass 392` re-attempted 2026-08-19, still blocked**: `self-verification forbidden
  (actor matches filed_by)` (exit 1) — same sandbox-wide same-actor guard documented across this
  repo's entire bug-fix-registration backlog (tasks 254, 358, and 24 others, all confirmed blocked
  identically this same session). Structural sandbox limitation, not a content gap in this task —
  left at 🔬 Verifying, not forced or spoofed.
