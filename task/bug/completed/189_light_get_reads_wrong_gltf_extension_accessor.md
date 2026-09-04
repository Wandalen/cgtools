# BUG-189: `light_get` reads `KHR_lights_punctual` via the generic extensions catch-all, always returning `None`

- **Severity:** High (the entire node-level light-resolution feature never functions -- every
  Point/Direct/Spot light attached to a glTF node via a `KHR_lights_punctual` reference is
  silently dropped, unconditionally, for every asset, with no error surfaced anywhere)
- **state:** Completed
- **Affects:** Any glTF asset loaded through this crate's `loaders::gltf` path that attaches a
  light to a node via the (ordinary, spec-standard) `KHR_lights_punctual` node extension --
  effectively every glTF asset carrying punctual lights, since node-level attachment is the only
  mechanism the spec defines for placing a light in the scene
- **Component:** `module/helper/renderer` (`src/webgl/loaders/gltf.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Discovered while writing BUG-172's own regression test, in the same
  function (`light_get`, same file). Tightly coupled: BUG-172's direction-formula fix was real
  but functionally unreachable dead code until this bug's fix let the function ever return
  `Some` at all. Independent root cause from BUG-171/BUG-173 (same review area, task #98,
  disjoint code paths).

## Symptom

```rust
// pre-fix -- webgl/loaders/gltf.rs, light_get
let light_id = gltf_node.extensions()?
.get_key_value( "KHR_lights_punctual" )?.1
.get( "light" )?
.as_u64()?;
```

`gltf_node.extensions()` always returns `None` for the `"KHR_lights_punctual"` key specifically,
so `light_get` always short-circuits via `?` and returns `None` -- for every node, on every glTF
asset, regardless of whether that node actually carries a valid light reference.

## Impact

**Who is affected:** Any consumer calling `renderer::webgl::loaders::gltf::light_get` (the
per-node light-resolution step of the glTF loading pipeline) on any asset that attaches a light
via `KHR_lights_punctual` -- the spec's only mechanism for placing a punctual light in the scene
graph. `light_list_get` (parses the document-level `lights` array into domain `Light` values)
still works correctly, so lights are parsed but never actually attached to any node.

**What breaks:** No Point, Direct, or Spot light ever resolves through this path. The renderer
silently proceeds as if no node in the scene referenced a light at all -- no panic, no log, no
error return, just a scene with all its intended lighting missing.

**Magnitude:** Total and unconditional for this code path -- not input-dependent, not an edge
case. Every glTF asset using node-level `KHR_lights_punctual` (the only way the spec defines to
place a light) loses every one of its lights silently.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Empirical, while writing BUG-172's own regression test (`light_get_derives_direction_from_
rotation_not_translation`, `tests/gltf_light_parsing_test.rs`). The test's first `light_get(...)
.expect(...)` call panicked with `None` even though the fixture parsed and validated correctly
via `gltf::Gltf::from_slice`. Traced to the crate source directly rather than assumed: read
`gltf-1.4.1/src/scene/mod.rs`'s `Node::extensions()` (returns `self.json.extensions.as_ref()?
.others` -- extension data *unknown* to the crate) alongside `Node::light()` (a separate, typed
accessor, `#[cfg(feature = "KHR_lights_punctual")]`-gated, reading `self.json.extensions
.khr_lights_punctual` directly). Confirmed definitively by reading `gltf-json-1.4.1/src/
extensions/scene.rs`'s `Node` extensions struct: `khr_lights_punctual` is a named field
(`#[serde(rename = "KHR_lights_punctual")]`), and `others` is `#[serde(flatten)]` -- flatten only
captures keys left over *after* every named field has claimed its own, so a named field's key can
never appear in the flattened catch-all. `renderer/Cargo.toml` confirms both the `extensions` and
`KHR_lights_punctual` Cargo features are enabled on the `gltf` dependency (line 62), so this is
not a configuration-dependent maybe -- it's the crate's guaranteed behavior for this workspace.
Cross-checked against `light_list_get` (two functions up in the same file), which already
correctly uses the equivalent document-level typed accessor (`gltf.lights()`) and whose tests
pass -- confirming the typed-accessor pattern is the established correct one, and `light_get`
was the sole outlier still using the generic catch-all for a extension the crate has typed
support for.

## Minimum Reproducible Example

```rust
// module/helper/renderer/tests/gltf_light_parsing_test.rs -- light_get_resolves_node_level_light_reference
let gltf = gltf::Gltf::from_slice( NODE_WITH_LIGHT_FIXTURE.as_bytes() ).unwrap(); // 1 node, KHR_lights_punctual -> light 0
let gltf_node = gltf.nodes().next().unwrap();
let node = Node::new();
let mut lights = FxHashMap::default();
lights.insert( 0, Light::Point( PointLight { position: F32x3::from_array([0.0;3]), color: F32x3::from_array([1.0;3]), strength: 1.0, range: 10.0 } ) );

let resolved = light_get( &gltf_node, &node, &lights );
assert!( resolved.is_some() ); // pre-fix: always None
```

**Expected** (post-fix): `Some(Light::Point(..))`, the light the node's extension references.

**Actual** (pre-fix): `None`, unconditionally -- confirmed directly by running this exact test
against the pre-fix production code (it panicked at the `.expect(...)` before any fix was
applied, which is how this bug was found in the first place).

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run -p renderer gltf_light_parsing_test::light_get_resolves_node_level_light_reference
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `light_get` reads the node-level `KHR_lights_punctual` reference via `gltf_node.extensions()`, the generic catch-all for extension data unknown to the crate -- but this crate deserializes `KHR_lights_punctual` into a named, typed field instead, so it's excluded from that catch-all by construction, making `light_get` always return `None`. | ✅ Root Cause | Confirmed by reading `gltf-json-1.4.1`'s `Node` extensions struct directly: `khr_lights_punctual` is a named `#[serde(rename)]` field, `others` is `#[serde(flatten)]` (only catches leftover keys); confirmed the relevant Cargo features are enabled in `renderer/Cargo.toml`; reproduced via a real failing test against pre-fix production code. | E1, E2, E3 |
| H2 | This is a test-fixture-only problem (my new fixture's JSON shape is wrong), not a production defect. | ❌ Falsified | The fixture parses and validates successfully via `gltf::Gltf::from_slice` (confirmed once the document-level `lights` array was added to satisfy the crate's own cross-reference validation); the failure is `light_get` itself returning `None`, traced to source, not a fixture/validation error. | E1, E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `~/.cargo/registry/.../gltf-1.4.1/src/scene/mod.rs` (`Node::extensions()` vs `Node::light()`) | `.extensions()` returns only `self.json.extensions.as_ref()?.others`; `.light()` is a separate, typed, feature-gated accessor reading `.khr_lights_punctual` directly. | H1 ✅ |
| E2 | `~/.cargo/registry/.../gltf-json-1.4.1/src/extensions/scene.rs` (`Node` extensions struct) | `khr_lights_punctual: Option<...>` is a named `#[serde(rename = "KHR_lights_punctual")]` field; `others: Map<String,Value>` is `#[serde(flatten)]` -- structurally excludes any named field's key. | H1 ✅ |
| E3 | `module/helper/renderer/Cargo.toml:62` | `gltf = { ..., features = [ "import", "extensions", "KHR_materials_specular", "KHR_lights_punctual", "utils" ], ... }` -- both relevant features unconditionally enabled. | H1 ✅ |
| E4 | `module/helper/renderer/tests/gltf_light_parsing_test.rs::light_get_resolves_node_level_light_reference` (real `cargo nextest` output, pre-fix and post-fix) | Pre-fix: panics with `None`. Post-fix: resolves `Some(Light::Point(..))` correctly. | H2 ❌ |

## Root Cause

```rust
// before -- reads the wrong accessor, always None for this key
let light_id = gltf_node.extensions()?
.get_key_value( "KHR_lights_punctual" )?.1
.get( "light" )?
.as_u64()?;
```

`gltf::Node::extensions()` is explicitly documented (in the crate's own source) as "extension
data unknown to this crate version." `KHR_lights_punctual` is *known* to this crate (typed
support is compiled in, gated by the same-named Cargo feature this workspace already enables),
so its data is deserialized into a dedicated field and never appears in the generic catch-all.
`light_list_get`, two functions above in the same file, already uses the correct pattern for the
equivalent document-level lookup (`gltf.lights()`); `light_get`'s node-level lookup was the sole
remaining site still reaching for the generic, "unknown extension" path for a known one.

## Why Not Caught

No existing test exercised `light_get` at all prior to this session -- the pre-existing 4 tests
in `gltf_light_parsing_test.rs` all test `light_list_get` (the document-level lights array
parser), never the per-node resolution step. `light_get` had zero test coverage of any kind until
BUG-172's own regression test was written this session, which is what surfaced this bug.

## Fix Location

`module/helper/renderer/src/webgl/loaders/gltf.rs`: `light_get` now resolves the light index via
the crate's own typed accessor:

```rust
// after
let light_id = gltf_node.light()?.index();
```

This also required widening `light_get`'s `lights` parameter from a concrete `&FxHashMap<usize,
Light>` to a hasher-generic `&std::collections::HashMap<usize, Light, S>` (`S: BuildHasher`) --
promoting the function to `pub` (done as part of BUG-172's own testability work, same session)
surfaced a pre-existing `clippy::implicit_hasher` lint that only fires on public API surfaces;
`FxHashMap` callers remain source-compatible with no changes needed at any call site.

## Prevention

Native regression test added: `light_get_resolves_node_level_light_reference`
(`tests/gltf_light_parsing_test.rs`) -- builds a minimal synthetic glTF document with one node
referencing a document-level light by index, and asserts `light_get` actually resolves it,
independent of any direction/rotation concern (that's BUG-172's own, separate test). No GL
context needed: `light_get` is pure CPU-side resolution logic once a `gltf::Node` and a
pre-built `lights` map are in hand.

## Pitfall

A crate that offers both a generic "extensions unknown to me" catch-all *and* a dedicated typed
accessor for one *specific*, well-known extension makes the catch-all silently exclude that
extension's data the moment the typed-support feature is compiled in. Reaching for the generic
path out of habit (or because a sibling extension genuinely has no typed support and must go
through it) fails silently -- `None`, never a compile error -- for exactly the well-supported
extensions a consumer is most likely to assume "of course the generic path covers this too."
Always check for a dedicated typed accessor first; the generic catch-all is a fallback for
extensions the crate does *not* know about, not a universal read path.

## Generalized Version

**Broken assumption:** "a generic `extensions()`/`extension_value()`-style catch-all accessor
returns *all* extension data present in the document, including extensions the library itself
has first-class typed support for."

**Confirmed general rule:** in any library that offers both a generic "unknown data" catch-all
and typed accessors for specific known cases (a common pattern beyond glTF -- e.g. `serde`'s own
`#[serde(flatten)]`), the catch-all is defined *in opposition to* the typed cases, not inclusive
of them. Always prefer the typed accessor when one exists; treat the catch-all as coverage only
for the residual, unrecognized case.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Discovered while writing BUG-172's own regression test in the same function; confirmed via a real failing test and direct reading of the `gltf`/`gltf-json` crate source. |
| 2026-08-16 | fixed | `light_get` resolves the light index via `gltf_node.light()?.index()` (the crate's typed accessor) instead of manually re-parsing the generic extensions catch-all; `lights` parameter widened to a hasher-generic `HashMap` to satisfy `clippy::implicit_hasher` on the now-`pub` function. |
| 2026-08-16 | verified | Native `cargo nextest -p renderer --all-features`: 91/91 passed (including this bug's own regression test and BUG-172's, which depends on this fix to ever reach its own assertions); `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the MRE directly from the actual failing test output (not reasoning-derived). Adversarial pass checked whether the failure could instead be a fixture-validation artifact (H2) -- ruled out by confirming the document parsed/validated cleanly and the panic traced to `light_get`'s own early return, verified against real crate source (`gltf-1.4.1`, `gltf-json-1.4.1`) rather than assumed from the method name alone. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-172 (same function, dependency relationship documented both directions) and BUG-171/BUG-173 (same review area, confirmed disjoint). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct source reads of both the consuming crate's method bodies and the underlying `gltf-json` struct definitions (`#[serde(flatten)]` vs named field), plus workspace `Cargo.toml` feature confirmation -- not inferred from behavior alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is a one-expression replacement inside `light_get` plus a signature generalization forced by the resulting `pub`-function clippy lint; no broader refactor attempted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `renderer`'s `src/webgl/loaders/gltf.rs`, its own test file, and this bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via source read that `light_list_get` (the only other `KHR_lights_punctual` consumer in this file) already uses the correct typed accessor -- no sibling occurrence of this defect pattern remains in the file. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix is purely an accessor correction; `light_get`'s own responsibility (resolve a node's light reference into a domain `Light`) is unchanged. The hasher-generic signature widening is a minimal, mechanical response to a clippy lint on the now-public function, not a scope expansion. | — |

**Reproduced:** YES -- a real native test against actual pre-fix production code panicked with
`None` at the exact `light_get` call site; the same test, run again after the fix with no other
change, resolves `Some(Light::Point(..))` correctly. Full scoped suite (91/91) and clippy both
clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/loaders/gltf.rs` | `light_get` resolves the light index via `gltf_node.light()?.index()` instead of manually parsing `gltf_node.extensions()`'s generic catch-all (full `Fix(BUG-189)` comment block); `lights` parameter widened from `&FxHashMap<usize, Light>` to a hasher-generic `&std::collections::HashMap<usize, Light, S>` to satisfy `clippy::implicit_hasher` on the now-public function. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/gltf_light_parsing_test.rs` | New `light_get_resolves_node_level_light_reference`: builds a minimal synthetic glTF document with one node referencing a document-level light by index, hand-builds the `lights` map `light_get` consumes, and asserts resolution succeeds -- independent of BUG-172's own direction-formula concern. |
