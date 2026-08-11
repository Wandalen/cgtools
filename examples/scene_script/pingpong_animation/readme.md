# Pingpong Animation

**Keywords:** Rhai, Scripting, Animation, Tweening, scene_script

This demo runs a Pong-style scene entirely from a `.rhai` script — loops, branches, and `F32x2` vector arithmetic simulate ball and paddle motion over 40 ticks, calling back into the host once per tick via a registered `emit_frame` function. It demonstrates the imperative half of `scene_script`'s two scripting patterns: a script driving the host by side effect, as opposed to a script that only builds and returns a value.

The host then takes two consecutive recorded frames and smoothly interpolates between them using `animation::Tween<F32x2>` with a `Linear` easing function — the real `animation` crate, not placeholder lerp math. This shows how a scripted simulation and the workspace's own animation/easing machinery compose: Rhai owns the per-tick logic, Rust owns the sub-frame interpolation.

An example-local `frame_to_commands` compiler (`src/render.rs`) then translates each recorded `Frame` into `tilemap_renderer::commands::RenderCommand`s — the ball and both paddles as mesh draws — per `docs/adr/003_d2_stack_hal_adoption.md` Decision #4 (glue code, not a shared crate: this is the only consumer). With the `adapter-svg` feature enabled, `main()` submits every frame's compiled commands to a `tilemap_renderer` `SvgBackend` and prints the resulting SVG's size. With the `adapter-webgl` feature enabled (wasm32 target only), `main()` instead submits them to a `tilemap_renderer` `WebGlBackend` bound to a browser-provided WebGL2 canvas via `minwebgl::context::retrieve_or_make`.

*(Console output by default; opt in to `--features adapter-svg` for SVG backend output via a plain native `cargo run`. `--features adapter-webgl` targets wasm32-unknown-unknown only — `tilemap_renderer`'s `WebGlBackend` needs a real browser-provided `WebGl2RenderingContext`, so a native run compiles but panics at runtime; see `src/main.rs`'s `render_frames` doc comment.)*

**[How to run](../../how_to_run.md)**

**References:**

* [Rhai Language Reference]
* [Custom Types in Rhai]

[Rhai Language Reference]: https://rhai.rs/book/
[Custom Types in Rhai]: https://rhai.rs/book/rust/custom-types.html
