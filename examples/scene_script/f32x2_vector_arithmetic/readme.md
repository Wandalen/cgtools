# F32x2 Vector Arithmetic

**Keywords:** Rhai, Scripting, Vector Math, scene_script

This demo runs a `.rhai` scene script that constructs two `F32x2` vectors and combines them with ordinary `+`/`*` operators, returning the result back to the host as a typed value. It demonstrates the declarative half of `scene_script`'s two scripting patterns: a script as a *data format*, building a value purely from expressions and returning it, as opposed to driving the host imperatively.

The `F32x2` type is registered into the `rhai::Engine` once, in Rust, via `Engine::register_type_with_name` + `Engine::register_fn` — its `+`/`-`/`*` operators dispatch straight to `ndarray_cg`'s own `std::ops` implementations rather than reimplementing vector arithmetic on the Rhai side. The host reads the script's return value back out with `engine.eval::<F32x2>(script)`, so the same type flows across the Rust/Rhai boundary in both directions without manual conversion.

*(No showcase — console/logic demo, no visual output)*

**[How to run](../../how_to_run.md)**

**References:**

* [Rhai Language Reference]
* [Custom Types in Rhai]

[Rhai Language Reference]: https://rhai.rs/book/
[Custom Types in Rhai]: https://rhai.rs/book/rust/custom-types.html
