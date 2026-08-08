# Pingpong Animation

**Keywords:** Rhai, Scripting, Animation, Tweening, scene_script

This demo runs a Pong-style scene entirely from a `.rhai` script — loops, branches, and `F32x2` vector arithmetic simulate ball and paddle motion over 40 ticks, calling back into the host once per tick via a registered `emit_frame` function. It demonstrates the imperative half of `scene_script`'s two scripting patterns: a script driving the host by side effect, as opposed to a script that only builds and returns a value.

The host then takes two consecutive recorded frames and smoothly interpolates between them using `animation::Tween<F32x2>` with a `Linear` easing function — the real `animation` crate, not placeholder lerp math. This shows how a scripted simulation and the workspace's own animation/easing machinery compose: Rhai owns the per-tick logic, Rust owns the sub-frame interpolation.

*(No showcase — console/logic demo, no visual output)*

**[How to run](../../how_to_run.md)**

**References:**

* [Rhai Language Reference]
* [Custom Types in Rhai]

[Rhai Language Reference]: https://rhai.rs/book/
[Custom Types in Rhai]: https://rhai.rs/book/rust/custom-types.html
