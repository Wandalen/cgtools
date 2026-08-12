# Pitfall: `rhai::serde`'s Bridge Requires the Exact `FLOAT` Type

### Scope

- **Purpose**: Warn a consumer deserializing a script's returned `Dynamic` value that `rhai::serde::from_dynamic` matches Rhai's own numeric type exactly — it does not narrow to whatever numeric type the target Rust struct declares.
- **Responsibility**: Document the concrete failure mode (a hard deserialization error, not silent truncation) and the established mitigation (declare the target field as `f64`, narrow explicitly after).
- **In Scope**: Numeric-type matching in `rhai::serde::from_dynamic`, as it affects any consumer that deserializes a `scene_script`-built engine's `Dynamic` return value into a Rust struct.
- **Out of Scope**: `scene_script`'s own source — the crate itself never calls `rhai::serde` (confirmed: no `serde` reference anywhere under `src/`); this trap belongs entirely to *consumers* that evaluate a script and extract structured data from its result, such as the orrery example (`examples/orrery/webgpu`).

### Trap

Rhai's `FLOAT` type is `f64` in this workspace — no crate enables `rhai`'s
`f32_float` feature (confirmed: `[workspace.dependencies.rhai]` in the root
`Cargo.toml` declares no features beyond what each consumer adds; neither
`scene_script`'s `["internals"]` nor `orrery_webgpu`'s `["serde"]` touches
float width). `rhai::serde::from_dynamic` bridges a `Dynamic` value into any
`serde::Deserialize` type, but it matches the dynamic value's *actual*
runtime type exactly — it does not perform the numeric narrowing/widening a
human reader might expect from "it's just a float." A Rust struct field
declared `f32` where the script produced an `f64` fails to deserialize at
all; it does not silently truncate to the nearest `f32`.

### Failure

Reproduced directly against this workspace's own code (not assumed):
temporarily changing `examples/orrery/webgpu/src/scene.rs`'s
`Color( pub f64, pub f64, pub f64 )` to `Color( pub f32, pub f32, pub f32 )`
and running `scene_rhai_parses_and_matches_known_values` produces an
immediate panic, not a truncated/approximate value:

```
scene.rhai's returned value must match SceneConfig's shape: ErrorMismatchOutputType("f32", "f64", none)
```

This is `SceneConfig::load()`'s own `.expect(...)` firing on
`rhai::serde::from_dynamic`'s `Err` — a build-time authoring mistake caught
loudly, per that function's own documented intent, not a runtime data
surprise. The failure is total (every color and scalar field in the struct,
since all of them cross the same boundary), not partial.

### Mitigation

Declare every field a script's `Dynamic` output feeds into as `f64`,
matching Rhai's `FLOAT` exactly, and narrow to `f32` explicitly afterward at
the point where a narrower type is actually needed (GPU uniform buffers,
`vec4f`-packed layouts, etc.) — this is exactly
`examples/orrery/webgpu/src/scene.rs`'s structure:
`Color( pub f64, pub f64, pub f64 )` deserializes directly from the script,
and [`Color::to_array`](../../../../../examples/orrery/webgpu/src/scene.rs)
performs the narrowing cast (`self.0 as f32`, ...) as a separate, explicit
step once the value is safely in hand. Never declare a deserialize-target
struct field at the narrower type "to save the cast" — the cast has to
happen somewhere, and doing it inside `serde` instead of after it turns a
one-line `as f32` into a hard deserialization failure.

### Patterns

| File | Relationship |
|------|--------------|
| [../../../../../docs/pattern/004_script_as_data.md](../../../../../docs/pattern/004_script_as_data.md) | This boundary is exactly where a script-as-data document's values cross from Rhai's type system into Rust's |

### Sources

| File | Relationship |
|------|--------------|
| `examples/orrery/webgpu/src/scene.rs` | `Color`, `SceneConfig` and friends — the worked example of the `f64`-in/`f32`-out mitigation; `SceneConfig::load()` is where the failure surfaces |

### Tests

No dedicated regression test pins this — it is a fact about `rhai::serde`'s
own bridging behavior, not a claim this workspace's code implements or
could regress on. `scene_rhai_parses_and_matches_known_values`
(`examples/orrery/webgpu/tests/scene_test.rs`) indirectly relies on the
mitigation being followed correctly (every field is already `f64`), but
does not test the mismatch failure itself; the exact error text above was
captured once, ad hoc, by temporarily reverting the mitigation and
observing the real panic (see Failure above).
