# Invariant: WebGL Bind-Group Entry Order

### Scope

- **Purpose**: Guarantee a texture-plus-sampler pair bound together in one WebGL bind group resolves correctly, since the WebGL backend pairs each sampler with the nearest preceding texture entry rather than an explicit pairing field.
- **Responsibility**: Document the entry-order contract on the slice passed to `bind_group_layout_create`/`bind_group_create` and why WebGL depends on it.
- **In Scope**: Texture-before-sampler ordering within one group's entries.
- **Out of Scope**: Call order of `RenderPass` recording methods during a pass (see `invariant/002`).

### Invariant Statement

Within one `bind_group_layout_create`/`bind_group_create` entry list targeting the WebGL backend, a `BindingType::Sampler` entry must be preceded by the `BindingType::Texture` entry it is meant to pair with, within the same group. Per the source's own wording: "A `Sampler` entry pairs with the nearest preceding `Texture` entry of its group."

### Enforcement Mechanism

Not compiler-enforced — a documented construction-order contract on the entries slice, not a type that pairs texture and sampler explicitly. `webgl.rs`'s `RenderPipelineWebGl` doc comment states the pairing rule directly, alongside the binding-name convention it rides on (uniform block `ub_{group}_{binding}`, sampler uniform `tex_{group}_{binding}`). The crate's own test suite encodes this explicitly: `tests/native_backend_test.rs`'s `textured_bind_group_create` helper builds its entries texture-first, with an inline comment calling the order "load-bearing" for the WebGL backend, even though the test itself runs under native (which binds explicitly and is unaffected by entry order).

### Violation Consequences

A `Sampler` entry with no preceding `Texture` entry in its group — or ordered after a different group's texture — pairs against the wrong texture, or against none, under the WebGL backend specifically. WebGPU and native, which bind explicitly by index rather than by adjacency, are unaffected by entry order.

### Features

| File | Relationship |
|------|--------------|
| [feature/004_bind_groups_and_layouts.md](../feature/004_bind_groups_and_layouts.md) | This feature's entry slice is exactly what this invariant constrains |

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl.rs` | `RenderPipelineWebGl` doc comment: binding-name convention and nearest-preceding-texture pairing |

### Tests

`tests/native_backend_test.rs::texture_write_readback` constructs its bind group in the required order (`textured_bind_group_create`'s inline comment names the order "load-bearing"), but the test runs under the native backend, so it does not itself exercise the WebGL-specific pairing logic this invariant protects against.
