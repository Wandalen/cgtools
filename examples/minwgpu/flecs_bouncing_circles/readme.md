# Flecs Bouncing Circles (wgpu)

**Keywords:** wgpu, flecs, ECS, Rust, 2D Physics

This demo drives a basic 2D physics simulation with the flecs Entity Component System and renders the result with wgpu. Ten circles with distinct colors, radii, positions, and initial velocities fall under gravity, collide with each other, and bounce off the arena walls, all with a fixed restitution coefficient.

Two flecs systems own gravity and the walls: `GravityIntegrate` accelerates and moves every circle each frame, while `WallBounce` reflects velocity when a circle's edge crosses the arena bounds. Circle-circle overlaps are resolved by a third step, `circles_collide` — a plain function rather than a system, since its pairwise scan needs every circle's state gathered before any of it is resolved, which a system's one-row-at-a-time callback can't express — called once per frame right after the two systems run, pushing overlapping circles apart and applying a restitution-scaled velocity response along the collision normal. Circle instance data (center, radius, color) is queried from the ECS world each frame and uploaded as a GPU instance buffer; every circle is drawn as a single SDF-shaded quad instanced across all entities. The example opens a real window and renders continuously, stepping the simulation each frame with `dt` taken from the real time elapsed since the previous frame (clamped to a ceiling so a stall produces one slow-motion frame rather than a large simulation jump). Close the window to exit.

![image](showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [flecs Documentation]
* [wgpu Documentation]

[flecs]: https://www.flecs.dev/flecs/
[flecs Documentation]: https://docs.rs/flecs_ecs/latest/flecs_ecs/
[wgpu Documentation]: https://docs.rs/wgpu/latest/wgpu/
