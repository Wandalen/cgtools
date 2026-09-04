# Raycaster

**Keywords:** Ray Casting, Collision, WebGL2, Interaction

This demo demonstrates ray casting for 3D scene interaction in WebGL2. Ray casting converts 2D screen coordinates to 3D rays, enabling precise object selection, collision detection, and physics interactions.

Ray casting is fundamental for 3D user interfaces and game mechanics. This example provides a reusable implementation for mouse-based 3D interaction.

![image](showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [Lode's Computer Graphics Tutorial] - Comprehensive raycasting guide
* [Wolfenstein 3D Source Code] - Original implementation
* [Ray Casting Computer Graphics] - Wikipedia overview

[Lode's Computer Graphics Tutorial]: https://lodev.org/cgtutor/raycasting.html
[Wolfenstein 3D Source Code]: https://github.com/id-Software/wolf3d
[Ray Casting Computer Graphics]: https://en.wikipedia.org/wiki/Ray_casting

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| src/main.rs | WebGL2 render loop: raycasting, minimap draw, per-frame input/movement wiring |
| src/controls.rs | Keyboard move/rotation direction state via `browser_input` |
| src/sim.rs | Pure map/raycasting/movement logic, kept dependency-free so `tests/` can exercise it natively |
| tests/wall_tunnel_test.rs | Regression coverage for `src/sim.rs` ( includes `bug_reproducer(BUG-522)` ) |
| Cargo.toml | Crate manifest: minwebgl/mingl/browser_input dependencies |
