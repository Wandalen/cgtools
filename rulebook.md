# Workspace Rulebook

Workspace-wide conventions for `cgtools`. This is the single source of truth
for repository-level lint and style rules; every crate in the workspace
follows the rules below unless an explicit, well-justified crate-local
override is recorded under the crate's own roof.

---

## Documentation layout

**Rule:** Each crate's design documentation lives in `docs/`, organized as
typed doc definition instances (`docs/feature/`, `docs/invariant/`, `docs/api/`,
and other types as warranted by content) per the standard doc definition
framework in `doc_des.rulebook.md`. Do not use a monolithic `spec.md` file
at the crate root.

Applies to all crates in the workspace that carry design documentation.

Companion files per crate:

- `docs/` — requirements, architecture, and design documentation as typed
  doc instances; present only when the crate has content that warrants it
  (see doc_des.rulebook.md's Documentation Necessity Test)
- `roadmap.md` — future work
- `readme.md` — user-facing entry point, may link into `docs/`
- `rulebook.md` — crate-local lint/style rules **only when overrides are
  needed**; absent by default, since this workspace rulebook is authoritative

**Rationale:** Typed doc instances keep documentation navigable by design
dimension and cross-referenceable at instance granularity — a requirement,
an invariant, and an API contract each evolve independently and are easier
to keep current as separate, focused files than as sections buried inside
one growing document.

---

## Workspace-scope documentation

**Rule:** Design documentation whose subject spans multiple crates —
ecosystem architecture decisions, cross-crate design patterns, design
explorations — lives in `docs/` at the repository root, organized as typed
doc definition instances (`docs/adr/`, `docs/pattern/`, `docs/explorations/`,
and other types as warranted) under the same doc definition framework that
governs crate-level `docs/`. A crate's own `docs/` remains the home for
anything scoped to that crate alone; the workspace collection never
duplicates crate-level content, it references it.

**Rationale:** A multi-crate decision has no single crate under whose roof it
could live without privileging one crate's perspective. One workspace-root
collection gives such content a single authoritative home instead of
duplicating it across every affected crate.

---

## Rendering layer placement

**Rule:** Every rendering-ecosystem crate occupies exactly one rung of the
L0–L5 ladder below, or is explicitly listed beside it. A new rendering crate
must be placeable on this table before it is added, and each layer depends
only on the layer directly beneath it, with a drill-down handle to that
layer (see [docs/pattern/002](docs/pattern/002_strict_layering_one_step_drilldown.md)).
The authoritative per-layer contracts live in [docs/layer/](docs/layer/readme.md);
this table is the friendly orientation copy and is updated together with
those cards.

| Layer | In plain words | Crates today | Reserved slot |
|-------|----------------|--------------|---------------|
| L5 — scene script + runners | Scenes as scripts you can parse, interpret, and rerun — same script, same frames | `tilemap_scene` (compiled scenes), `scene_script` (Rhai glue) | `d3_scene` |
| L4 — scene model | What exists, as data files — loadable and checkable without any GPU | `tilemap_scene` (RON model); glTF via `renderer` loaders; procedural glTF assembly via `primitive_generation` | `d3_scene` |
| L3 — stack engine | Turns one stack's vocabulary into draw work; one engine per stack | `tilemap_renderer` (d2), `renderer` (d3) | — |
| L2 — frame orchestration | Which passes run, in what order, into which render targets | embedded in `renderer` and `tilemap_renderer` today | `frame_graph` |
| L1 — GPU abstraction | One GPU API over all backends, so code is written once per stack instead of once per backend | `gpu_hal` (v0: WebGPU + WebGL2 + native `wgpu` + native `vulkan`, serving `renderer`'s canonical path; `vulkan` implemented — see [ADR-004](docs/adr/004_native_vulkan_hal_backend.md)) | — |
| L0 — drivers | Thin Rust wrappers over the raw GPU APIs, one per backend | `minwebgl`, `minwebgpu`, `minwgpu`, `minvulkan` (real Instance/Device/Queue, tested against a live ICD; real surface/swapchain too — see [ADR-006](docs/adr/006_vulkan_windowed_presentation.md)) | — |
| (substrate) | Shared helpers the drivers build on — below the ladder, not a layer | `mingl` | — |

`line_tools` is not beside the ladder — it is already pinned twice, split by
submodule: `line_tools::d2` is a `d2`-stack crate, `line_tools::d3` is a
`d3`-stack crate (both already listed in
[docs/adr/001](docs/adr/001_multi_stack_rendering_architecture.md)'s own
per-stack tables — neither pending nor a single classification, since
neither module is an "engine" occupying the L3 row above; both are
line-geometry-and-shader utilities their respective stack's engine can call
directly).

Beside the ladder: `canvas_renderer` (cross-stack bridge via textures — see
[docs/pattern/003](docs/pattern/003_cross_stack_bridge_via_foundation_resources.md)),
`tiles_tools` (tile-logic library feeding L4),
`animation` (value interpolation, easing, and multi-animation sequencing —
feature-gated to `minwebgl`/`mingl`'s math/future/diagnostics utilities, not
their GL-context layers, so it is a horizontal capability rather than an
L0 occupant; feeds `scene_script`'s tween bindings today),
`shader_chunks_render_core` and `shader_chunks_preview_web` (headless and
browser shader-chunk authoring/preview tooling — single-backend by design
via direct `minwgpu`/`minwebgpu` dependencies, not a portability seam any
stack needs; see
[docs/layer/001](docs/layer/001_l0_drivers.md#non-stack-tooling-consumers)),
`gl_uniforms` (program-scoped WebGL uniform upload wrapper — a thin
ergonomic layer directly over `minwebgl`'s uniform primitives, not a
portability seam or orchestration layer, so it sits beside L0 rather than
occupying a rung),
`gpu_picking` (GPU id-buffer object picking — renders pickable parts into
an off-screen R32I texture and reads a pixel back to find what's there; a
thin, WebGL2-only capability directly over `minwebgl`, same shape as
`gl_uniforms`, not a portability seam or orchestration layer).

**Rationale:** One glance answers "where does my crate sit, and what may it
depend on" without walking the doc graph. The detailed contracts stay
single-sourced in `docs/layer/`, which keeps this table honest and one
screen tall.

---

## Test placement

**Rule:** Every test lives in its crate's `tests/` directory. No `src/` file
carries a `#[cfg(test)] mod tests { ... }` block, whatever the visibility of
what it tests.

A test that needs a private item reaches it through a `test_internals`
cargo feature, off by default:

```rust
// src/lib.rs
/// The internals `tests/` reaches, exposed only under `test_internals`.
///
/// Not part of the surface. With the feature off — the default, and what
/// every dependent gets — this module does not exist.
#[ cfg( feature = "test_internals" ) ]
#[ doc( hidden ) ]
pub mod internal
{
  pub use crate::private::{ some_helper, SomeType };
}
```

The item becomes `pub` *inside its private module* and is re-exported only
through that gate, so the default build's surface is byte-for-byte what it
was. `tests/` then runs under `--features test_internals`.

**Rationale:** A test's placement should be one rule, not a rule plus a
judgement about what the test happens to touch. The old form split them —
public-API tests in `tests/`, private-helper tests inline — and the split
cost more than it saved. A test migrates across the boundary as the code
under it changes visibility, so the correct location was a moving target;
`gpu_picking`'s pure predicates and `renderer`'s live-context reproducers sat
in `src/` for the same stated reason while being nothing alike; and a reader
looking for a crate's tests had to check two places and know which rule
applied to each.

The encapsulation objection the old rule raised is real but is answered by
the gate rather than by inline placement. `#[ doc( hidden ) ]` plus
off-by-default means no dependent sees the item, no documentation lists it,
and no semver promise attaches to it — the `pub`/`fn` distinction survives
intact for every consumer. What changes is only that the crate's own test
binary can opt in.

There is a second gain the old rule gave up. An integration test compiles
against the same surface a caller gets, so a round trip it exercises is a
round trip a caller could actually write — an inline test proves that only
by accident. Where a test genuinely needs the interior, the feature gate
makes that need explicit and greppable instead of invisible.

**Wasm tests:** `wasm_bindgen_test_configure!( run_in_browser )` must be
linked into a test binary at least once for that whole binary to run in a
browser rather than Node, where `web_sys::window()` is always `None`. Each
file under `tests/` is its own binary, so each wasm test file carries its
own call. A missing one does not fail to compile — it fails at runtime with
an unrelated-looking `CanvasRetrievingError("Failed to get window")` on
every test in the binary.

---

## Test file size

**Rule:** Test source files have **no fixed line-count limit** in this
workspace. Files SHOULD be split by domain (compile slice, feature area,
anchor kind, etc.) when one of the following triggers emerges:

- a contributor reports concrete navigation difficulty,
- incremental compile time on a single integration-test binary becomes a
  measurable bottleneck,
- a coherent sub-domain has grown large enough that a dedicated file would
  reduce cognitive load for future readers.

**Rationale:** Line count is a poor proxy for maintenance cost. Many crates
here share substantial fixture surface (spec / scene builders, mock data,
extractors) that is far cheaper to keep co-located in one file than to
mirror across many small files via `tests/common/mod.rs` plumbing.
Splitting on domain boundaries when a real pain point appears yields more
useful files than splitting on an arbitrary line threshold.

---

## `#![allow]` and `#[allow]` attributes in source files

**Rule:** File-level `#![allow(...)]` and item-level `#[allow(...)]`
attributes are **permitted** anywhere in this codebase.

**Rationale:** The workspace already sets `allow_attributes = "allow"` in
`[workspace.lints.clippy]`, acknowledging that targeted suppressions are a
legitimate tool. This repository uses proc-macros (`mod_interface!`, derive
macros, etc.) whose expansions can trigger lints at call sites where there
is no per-item scope available. Moving every suppression to the workspace
`Cargo.toml` would either (a) loosen lint policy globally across all crates,
or (b) require silencing lints in the workspace config that are
intentionally `warn` for most code.

Preferred suppression order (narrowest scope first):

1. Fix the code so the lint no longer fires.
2. `#[allow]` on the specific item if the warning is a false positive for
   that item.
3. `#![allow]` at file level if the warning is inherent to a macro
   expansion that affects the whole file (e.g. `mod_interface!` in
   `lib.rs`).
4. `[workspace.lints.clippy]` only when the suppression should apply
   crate-wide or workspace-wide by design.

Each `allow` attribute should have a short comment explaining why it is
needed.

---

## External path references

**Rule:** This repository's own tracked content (task files, docs,
rulebooks, source comments, scripts) must never contain an absolute or
relative path pointing to a file outside this repository. When something
genuinely needs to interoperate with content outside `cgtools` — tooling, a
relocated crate, a standalone deliverable — relocate the interoperating
piece to a neutral sibling location (a "yard" repo, e.g.
`~/pro/lib/yrd_gamedev/<name>`) rather than writing the external path into
this repository's own tracked files.

**Rationale:** A path into another repository is fragile — it breaks the
moment that repository moves, or for any clone/checkout at a different
location — and it silently couples this repository's own history to another
repository's layout. Routing interoperation through a neutral yard location
keeps `cgtools`' own tracked content fully self-contained. Worked precedent:
`task/cancelled/056_vectorizer_revival_watch_item.md` (a crate recovered
from history and relocated to the yard rather than referenced in place).

**Known exception:** `linter.rulebook.md § Layout : Standard Directory
Layout · LN001` requires a project's linter workspace to live at the
project root. This repository's linter (`cgtools_linter`) and its
`verb/lint` entry point deliberately violate that: both live in the yard
(`~/pro/lib/yrd_gamedev/linter/`, `~/pro/lib/yrd_gamedev/verb/lint`)
instead, because the linter inherently needs simultaneous access to both
itself and the sibling `cgtools/` repo it checks — nesting it inside either
repo would force exactly the cross-repo path reference this rule exists to
prevent.
