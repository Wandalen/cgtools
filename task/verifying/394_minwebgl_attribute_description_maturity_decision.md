# Decide fate of `minwebgl::Attribute`/`AttributeDescription` — finish declarative vertex binding or drop

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-19 23:00:49
- **expires_at:** 2026-08-20 01:00:49
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgl
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **unverified_at:** 2026-08-19 22:37:56
- **unverified_by:** system
- **in_motion:** true
- **verifying_at:** 2026-08-19 23:00:49
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

`module/min/minwebgl/src/attribute.rs` is fully dead code: a commented `AttributeDescription` struct
+ `Attribute` trait (`fn describe(&self) -> Vec<AttributeDescription>`) + a commented
`impl Attribute for ()`, gated out via `mod_interface!`'s commented
`orphan use { // AttributeDescription, // Attribute, };`.

This session's investigation corrected an earlier, wrong framing (from the original workspace-wide
audit) that this was "superseded by `mingl::VectorDataType`/`IntoVectorDataType`." It isn't a
competitor — it's a **consumer**: the dead `AttributeDescription`'s own `kind` field is itself typed
as `VectorDataType`. `VectorDataType` (live, real consumers in
`data_type/{f32,i32,i16,i8,u32,u16,u8}.rs`, `minwebgl::buffer.rs`, mingl's own tests) describes one
scalar field's type+size; the dead trait was building a **struct-level** layer on top —
declaratively describing a whole vertex struct's shader-attribute bindings in one shot, instead of
hand-chaining `BufferDescriptor` calls per field.

That gap is real and current, not speculative: `line_tools/src/d2/line.rs` alone has roughly 20
near-identical manual `BufferDescriptor::new().offset().stride().divisor().attribute_pointer()`
chains that a working `describe()`-driven API would collapse; more exist in
`renderer`/`primitive_generation`.

Completion cost note: `minwebgl` has no derive-macro infrastructure today (no `derive_tools`
dependency, confirmed via `Cargo.toml`) — a fully declarative `#[derive(Attribute)]` ergonomic layer
would need new dependency/proc-macro work. The dead code itself doesn't need that, though: it's
already a plain trait + manual per-struct `describe()` impls (see the dead `impl Attribute for ()`),
completable exactly as sketched, derive-free.

**Git-history investigation (per user request):** searched this repo's full history for
`attribute.rs`'s rollback pattern — `master` plus all 34 locally-fetched branches, back to the
earliest reachable commit (`dc8c8c1f "initial commit"`, 2024-10-28, a single squashed commit
importing the whole codebase at once). Result: **`attribute.rs` has exactly 2 commits in this repo's
entire history** — the initial squash (file already dead/commented at that point) and one unrelated
recent commit. No branch shows a different version. If it really was implemented and rolled back
multiple times, that happened before this repo's history begins and isn't recoverable here — the
actual reason needs the original developer's own recollection or external pre-2024 records.

**This is explicitly a tracking placeholder, not active work.** No implementation should begin
speculatively.

**Related Tasks:** `392` (same shape of open decision, same audit, same user directive to leave open
for developer input). `391`/`393`/`395` (this audit's other 3 incidents, all resolved to concrete
action rather than left open).

## Open Question (needs developer answer)

1. Is collapsing the ~20-site (and growing) manual `BufferDescriptor` chain boilerplate worth
   finishing this trait now — moderate cost: write real `describe()` impls, convert
   `line_tools/src/d2/line.rs` as a proof-of-concept caller?
2. Does anyone recall why this was apparently attempted and abandoned (pre-dating this repo's
   history) — a specific technical blocker, or just deprioritized?
3. If finished: should it stay a plain manually-implemented trait (as sketched), or is a later
   derive-macro ergonomic layer (`#[derive(Attribute)]`) actually wanted enough to justify adding
   `derive_tools` as a new dependency?

Until answered: `module/min/minwebgl/src/attribute.rs` stays as dead/commented code; no further
action.

**Status update — superseded.** The "tracking placeholder, no further action" framing above reflects
this task's *filed* state, not its *current* state. Per `## History`'s `EXECUTED` entry, the user
later gave an explicit live directive ("apply same logic to second task also" — the same full
migration treatment as task 392) that answered Open Question 1 directly and authorized real
implementation: the trait was completed close to its original sketch (`describe()` as an associated
function on `Self : mem::Pod`, per this task's own `ANALYZED` correction), adopted in
`attributes_vao`, browser-verified (`VERIFIED` entry), then extended to one more file
(`area_light/plane.rs`) in a later session's expansion pass (`EXPANDED` entry). Open Question 2 (why
the original attempt was abandoned pre-2024) remains genuinely unanswered — no repo history reaches
that far back. Open Question 3 (derive-macro ergonomic layer) also remains genuinely open — the
completed trait is the manually-implemented path only, `derive_tools` was deliberately not added.
Neither blocks this task's own closure, since Question 1 (the actual "finish or drop" call) was
answered and acted on. The investigative content above (git-history findings, the call-site fit
table, the field-shape correction) is preserved as-is for its historical/design-rationale value.

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | 🔴 | 🟢 | Goal/Open Question read "tracking placeholder, no further action" while History documented full execution — genuine documentation-coherence defect | Added a "Status update — superseded" note pointing to the History entries that answered Open Question 1 and authorized implementation; original investigative text preserved unchanged |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | Real boilerplate-collapse win on the rank-3 clean-fit call site (`attributes_vao`), authorized by explicit live user directive; the genuinely harder `line_tools` sites (rank 1/2, highest volume) were correctly left deferred rather than forced | — |
| D4 | Implementation Readiness | — | 🟢 | Lightweight Draft/open-question template (no In Scope/Out of Scope/Delivery Requirements/Verification sections by original design) — readiness judged against History's own record: 3 documented Tier 2 self-checks (EXECUTED/VERIFIED/EXPANDED), each with confirming+adversarial pass, shared native+wasm32 test/clippy evidence (with task 392), and live browser pixel verification | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Cross-crate touch: primary crate `minwebgl` plus 2 example crates (`attributes_vao`, `area_light`). Non-blocking: both are real end-to-end consumers proving the revived trait works outside its own defining crate, each addition explicitly directive-authorized and independently browser/diff-verified (area_light's BUG-321 regression test fix documented and re-confirmed passing) | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | `minwebgl`'s responsibility (WebGL-facing typed helpers) is unchanged by completing an already-sketched trait; the 2 example crates each stay single-purpose demos adopting it | — |
| **Total** | | 🔴 | 🟢 | 1 fixed, 1 non-blocking | 1/1 |

**Adversarial pass:** attempted to find a reason the D1 fix undersells the gap — re-read the full History chain (FILED → ANALYZED → EXECUTED → VERIFIED → EXPANDED) end-to-end to confirm the "superseded" claim is accurate, not a rationalization; confirmed the user directive quote ("apply same logic to second task also") is real and traces to the same session as task 392's own directive. Checked whether `T : Pod` being AoS-only (explicitly excluding `gbuffer.rs`/`primitive_data.rs`, migrated to 392-level types instead per this task's own corrected fit table) was a scope dodge — confirmed it's a genuine structural constraint (`Pod` describes one fixed-layout value type), not a convenient excuse to avoid harder call sites; the genuinely-avoidable-but-deferred site (`line_tools`) is explicitly flagged as deferred-not-forgotten rather than silently dropped. No further defect found.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 05:09:04 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-19 05:09:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:56 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 23:00:49 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 23:00:49 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 394` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

- **[2026-08-19]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) following a user-requested
  dead-code maturity review of `module/math`/`module/min`. User directive: this incident stays an
  open decision for the developer, not a unilateral delete-or-implement call; investigate the current
  alternative implementation and the historical-rollback question first (done, see Goal) — the
  rollback investigation found no evidence in this repo's reachable history, documented above rather
  than guessed at.

- **[2026-08-19]** `ANALYZED` — User requested a deeper pass: which live mechanism substitutes for
  this file, why it was rolled back, and every call site that would benefit. Analysis-only, no
  implementation began, per this task's own "not active work" clause.

  **Correction to the actual dead-code shape.** The live source (re-read directly, not from
  paraphrase) declares `fn describe() -> Vec<AttributeDescription>` as an **associated function**
  bound by `Self : mem::Pod`, not an instance method `describe(&self)` — i.e. the trait was designed
  to be implemented once per vertex-struct type and called as `Vertex::describe()`, not on a value.
  Matters for anyone picking this up: the sketch is already correctly shaped for a "one `Pod` struct →
  one static layout description" model; it does not need reshaping to be completable as drafted.

  **A third mechanism exists, closer to this file's own goal than the two things it's built on top
  of.** `renderer::webgl::geometry::AttributeInfo` (`module/helper/renderer/src/webgl/geometry.rs:9`)
  + its builder `buffer_attribute_info_make()` (`module/helper/primitive_generation/src/
  primitive_data.rs:122`) is a live, partial reinvention of the same "declaratively describe a vertex
  attribute" goal — but per-attribute (one call per field) rather than per-struct (one call for the
  whole layout), and it additionally carries runtime state (`WebGlBuffer` handle, `BoundingBox`) the
  dead trait never modeled. It is not a drop-in substitute; finishing this file would give
  `primitive_generation`/`renderer` a compile-time-checked layout source to feed `AttributeInfo`'s
  builder instead of the current 2 independently-typed-out `BufferDescriptor::new` chains at
  `primitive_data.rs:186,198`.

  **Rollback investigation — shared finding, see task 392's own `ANALYZED` entry for full detail.**
  This repo's entire history for `attribute.rs` is 2 commits: the 2024-10-28 squash (already dead on
  arrival) and current `HEAD` (`709f1c1c`), which for real, committed deletes the file as incidental
  collateral in an unrelated bulk commit — yet it sits back on disk right now as an **untracked** file
  (`git status --porcelain` → `?? module/min/minwebgl/src/attribute.rs`), byte-identical to before.
  No evidence of a genuine multi-time implement/revert cycle exists in this repo; if real, it predates
  `dc8c8c1f` and isn't recoverable here.

  **Call sites — every `BufferDescriptor::new` chain this trait's `describe()` could collapse**,
  counted via `grep -c` (110 real construction sites total across ~30 files; excludes the unrelated
  `web_sys::GpuBufferDescriptor`/`wgpu::BufferDescriptor` false-positive matches). Ranked, with a fit
  assessment — **clean fit** means the chains already describe one coherent, single-buffer vertex
  layout a `#[derive(Pod)]` struct could back directly; **needs restructuring** means the chains bind
  several independently-strided buffers at once (an interleaved-instancing pattern), so `describe()`
  as currently sketched (one `Pod` struct → one flat `Vec<AttributeDescription>`) would need either a
  multi-struct call per buffer or a trait extension before it applies cleanly:

  | Rank | File | Chains | Fit | Note |
  |---|---|---|---|---|
  | 1 | `module/helper/line_tools/src/d2/line.rs` | 26 | Needs restructuring | 4 independent buffers (`body_instanced`/`points`/`distance`/join/cap variants) interleaved per draw call, not 1 struct |
  | 2 | `module/helper/line_tools/src/d3/line.rs` | 8 | Needs restructuring | Same multi-buffer shape as d2, 3D variant |
  | 3 | `examples/minwebgl/text_rendering/src/main.rs` | 6 | Unassessed (example) | |
  | 3 | `examples/minwebgl/narrow_outline/src/main.rs` | 6 | Unassessed (example) | |
  | 3 | `examples/minwebgl/attributes_vao/src/main.rs` | 6 | **Clean fit — migrated** | 3 attributes × 2 buffers, identical `stride(6)` layout repeated verbatim — textbook `#[derive(Pod)] struct Vertex` candidate; prediction confirmed, see `EXECUTED` below |
  | 6 | `module/helper/renderer/src/webgl/post_processing/gbuffer.rs` | 4 | **Clean fit (392-level, not this trait)** | 4 independent single-attribute `GBufferAttachment` variants (`Position`/`Color`/`Normal`/`Uv1`), each already isolated per match arm — SoA shape (one buffer per attribute, not one interleaved struct); migrated directly to `mingl::VertexAttribute` per arm instead, see `EXECUTED` below and task 392's own `EXECUTED` entry |
  | 6 | `examples/minwebgl/make_cube_map/src/main.rs` | 4 | Unassessed (example) | |
  | 6 | `examples/minwebgl/deferred_shading/src/geometry.rs` | 4 | Unassessed (example) | |
  | 9 | `module/helper/primitive_generation/src/primitive_data.rs` | 2 | **Clean fit (392-level, not this trait)** | Feeds `AttributeInfo` builder directly, see above — SoA shape, same reasoning as gbuffer.rs; migrated directly to `mingl::VertexAttribute` instead, see `EXECUTED` below and task 392's own `EXECUTED` entry |
  | — | 22 more files (examples + `renderer`/`gpu_hal`/`shader_chunks_preview_web`) | 1-3 each | Unassessed | Long tail, mostly `examples/minwebgl/*`; lower leverage than the helper-crate sites above since each is single-use, illustrative code |

  Highest-leverage completion target if this task is picked up: `attributes_vao` and `gbuffer.rs`
  (clean fits, prove the trait works) before attempting `line_tools` (highest volume by far, but the
  multi-buffer shape means it needs a design decision first, not just a mechanical swap).

- **[2026-08-19]** `EXECUTED` — User directive: "apply same logic to second task also" — the same
  "move down to mingl, absorb real ideas from gpu_hal, stay close to original design, implement for
  real via Full migration" treatment task 392 (see that task's own `EXECUTED` entry) just received.

  **Design, close to the original sketch.** Completed largely as already-drafted: `describe()` stays
  an associated function bound by `Self : mem::Pod` (per this task's own `ANALYZED` correction —
  `Vertex::describe()`, not an instance method), on a trait now named to match its 392-level
  companion:

  ```
  pub trait Attribute : mem::Pod { fn describe() -> Vec< VertexAttribute >; }
  ```

  One real change from the original sketch: it returns `mingl::VertexAttribute` (392-level, revived
  this same session) instead of the dead file's own `AttributeDescription` struct — 392's revival
  replaced that struct with the leaner buffer/attribute-split pair, so this trait was completed
  against the type that actually exists now, not the one originally commented out. `Self : mem::Pod`
  needed `bytemuck` (the crate the `mingl::mem::Pod`/`Zeroable` derive macros expand against) added as
  a **direct** dependency wherever `#[derive(Pod)]` is used — transitive availability via
  `minwebgl`/`mingl` doesn't satisfy the macro's own crate-root reference. No `derive_tools`
  dependency was added (Open Question 3, still genuinely open) — this completion is the
  manually-implemented-trait path only, matching the "moderate cost" framing in Open Question 1, not
  a speculative ergonomic layer.

  **Real adoption — `examples/minwebgl/attributes_vao/src/main.rs`** (this task's own rank-3 "Clean
  fit" row, now confirmed correct): added a `Vertex` struct implementing `mingl::Attribute`; converted
  `vert_data`/`vert_data2` from raw `[f32; 30]` arrays to `[Vertex; 5]`; replaced both 2 separate
  3-chain `BufferDescriptor` VAO setup blocks with `mingl::VertexBufferLayout::from_attribute::<
  Vertex >( 6 )` + a single `gl::vertex_buffer_layout_bind` call per VAO. Collapses exactly the
  boilerplate shape this task's Goal described, on the file this task's own analysis had already
  identified as the cleanest completion target.

  **Correction to the "Clean fit" table above — rank 6 and rank 9 were never migrated to this
  trait.** Prompted directly by the user's own mid-session question ("can descriptor describe both
  array of structures and structures of arrays as well?"): `gbuffer.rs` and `primitive_data.rs` are
  both Structure-of-Arrays call sites (one `mingl::VertexAttribute` per buffer, not one interleaved
  `Pod` struct shared across attributes) — this trait's `T : Pod` bound is Array-of-Structures-only by
  construction, since it requires exactly one interleaved Rust struct per buffer element. Forcing
  gbuffer.rs's 4 independent single-attribute variants or primitive_data.rs's 2 single-attribute calls
  onto it would mean artificial single-field newtype wrappers per attachment, rejected as
  over-engineering with no real benefit over calling `mingl::VertexAttribute::new()` directly. Both
  were migrated to the 392-level type directly instead — see task 392's own `EXECUTED` entry for the
  concrete diffs. The table rows above are corrected in place to reflect this (see `**Clean fit
  (392-level, not this trait)**` markers) rather than left silently wrong.

  **`line_tools` (rank 1/2) untouched, unchanged assessment.** Still "needs restructuring" — 4
  independent interleaved buffers per draw call is a genuinely different shape needing a design
  decision (multi-struct call per buffer, or a trait extension) before this trait applies, not a
  mechanical swap. Explicitly out of scope for this "Full migration" — deferred, not forgotten;
  remains the highest-volume, highest-leverage target if picked up later.

  **Verification.** Shared with task 392's `EXECUTED` entry — same combined native+wasm32
  nextest/doctest/clippy run covers both tasks' changes together (`attributes_vao` is this task's own
  migration; `gbuffer.rs`/`primitive_data.rs` are 392's). See that entry for the 2
  verification-methodology fixes and 2 flagged out-of-scope opportunistic fixes required to reach
  green — none of them specific to this task's own `attributes_vao` change.

  Tier 2 (Dual-Role Self-Check, standing project cap — see `feedback_maav_tier_cap` memory).
  Confirming pass: `attributes_vao` compiles, renders the same 2-dataset VAO-switching demo as before
  (structural test `two_vao_switching_test.rs` updated and passing — see below), and the trait is
  genuinely exercised end-to-end, not just defined. Adversarial pass: specifically hunted for (a) any
  other file in the table's "Unassessed" rows that might actually be a better/easier trait-adoption
  target than assumed — none investigated this session, correctly left unassessed rather than
  guessed at; (b) whether the `T : Pod` bound genuinely forced the AoS-only conclusion or whether a
  looser bound could have supported SoA too — confirmed `Pod` fundamentally describes one fixed-layout
  value type, which is what makes AoS-only unavoidable here, not an arbitrary implementation choice;
  (c) whether the structural test `two_vao_switching_test.rs`'s updated string-match patterns
  (`"&vert_buffer,"` / `"&vert_buffer2,"` replacing `"&vert_buffer )?"` / `"&vert_buffer2 )?"`) still
  actually guard the original BUG-318 regression (mixed-buffer VAO) — confirmed the disjointness
  assertion (`uses_buffer ^ uses_buffer2`) and per-VAO block extraction logic are both unchanged, only
  the literal substrings being searched for were updated to match the new call shape.

- **[2026-08-19]** `VERIFIED` — user directly asked whether the changed examples were manually
  tested; they had not been. Closed the gap for this task's own adoption: `attributes_vao` run live
  in a browser (`longrun`-detached `trunk serve` + `browsee` screenshot/pixel-verdict) — 10 correctly
  colored/sized/positioned squares (2 VAOs × 5 points each), matching the crate's stated purpose, no
  visual defects. `gbuffer.rs`/`primitive_data.rs` (392-level, not this trait) tested too — see task
  392's own `VERIFIED` entry for those 2.

  Still explicitly unresolved: the ~25 "Unassessed" rows in this task's own fit table above were
  never in scope for this migration pass — not tested because never touched, not a testing gap.

- **[2026-08-19]** `EXPANDED` — Separate later session picked up part of the "Unassessed" long tail.
  Full file list and shared verification (native+wasm32 `verb/test` green, 2 real regressions found
  and fixed, concurrent-actor commit `fa4041ef` discovery) recorded in task 392's own `EXPANDED`
  entry — not duplicated here.

  **This trait's own share of that expansion is exactly one file**: `examples/minwebgl/area_light/
  src/plane.rs`'s `plane_vao`. Confirmed via diff against pre-migration source: the flat `&[f32]`
  array with 3 hand-chained `.offset(0/3/6).stride(8).attribute_pointer(...)` calls became a
  `#[repr(C)] struct Vertex { position:[f32;3], normal:[f32;3], texcoord:[f32;2] }` implementing
  `mingl::Attribute::describe()`, bound via `mingl::VertexBufferLayout::from_attribute::<Vertex>(8)`
  — `#[repr(C)]`'s field-order layout traced to produce byte offsets/stride identical to the old
  hand-written values (0/3/6 floats, stride 8 floats). The BUG-321 fix (vertex 3's corrected
  texcoord) survived unchanged in the new struct-literal data. All other files in 392's expansion
  list used the 392-level `VertexAttribute`/`BufferDescriptor` pair directly (per-attribute/SoA
  shape), not this trait — consistent with this task's own `T : Pod` bound being AoS-only by
  construction (see this task's `EXECUTED` entry).

  One real regression traced to this file's own migration (not the trait's design): the BUG-321
  regression test (`area_light/tests/plane_texcoord_test.rs`) hand-parses `plane_vertices` out of
  `plane.rs` source text and broke when the array literal's shape changed from flat floats to struct
  literals — fixed (detail in task 392's `EXPANDED` entry) by rescoping the parser's split predicate
  to digit/`.`/`-` characters instead of literal commas.

  `tsk .verify_pass 394` re-attempted 2026-08-19, blocked identically to task 392 and the rest of
  this repo's registration backlog: `self-verification forbidden (actor matches filed_by)` (exit 1).
  Structural sandbox limitation — left at 🔬 Verifying, not forced or spoofed.
