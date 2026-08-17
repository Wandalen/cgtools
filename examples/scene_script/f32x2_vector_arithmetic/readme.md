# F32x2 Vector Arithmetic

**Keywords:** Rhai, Scripting, Vector Math, scene_script

This demo runs a `.rhai` scene script that constructs two `F32x2` vectors and combines them with `+`/`*` operators, returning the result back to the host as a typed value. Structurally the script is declarative in *shape* — a `let`/`let`/trailing-expression sequence with no `main()`, loop, or branch — as opposed to `pingpong_animation`'s imperative shape, confined to `main()`. But the shape is not the whole story: every operator and the `F32x2` constructor dispatch to engine-registered bindings (see below), which makes this script `scene_script`'s script-as-glue pattern in substance despite its declarative-looking shape — see [pattern/005](../../../docs/pattern/005_script_as_glue.md)'s own worked analysis of this exact script for why declarative shape does not imply the script-as-data pattern.

The `F32x2` type is registered into the `rhai::Engine` once, in Rust, via `Engine::register_type_with_name` + `Engine::register_fn` — its `+`/`-`/`*` operators dispatch straight to `ndarray_cg`'s own `std::ops` implementations rather than reimplementing vector arithmetic on the Rhai side. The host reads the script's return value back out with `engine.eval::<F32x2>(script)`, so the same type flows across the Rust/Rhai boundary in both directions without manual conversion.

*(No showcase — console/logic demo, no visual output)*

**[How to run](../../how_to_run.md)**

**References:**

* [Rhai Language Reference]
* [Custom Types in Rhai]

[Rhai Language Reference]: https://rhai.rs/book/
[Custom Types in Rhai]: https://rhai.rs/book/rust/custom-types.html
