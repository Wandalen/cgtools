# Grid Rendering (wgpu)

**Keywords:** wgpu, Rust, Grids, Native Graphics

This demo renders a flower of 7 hexagons -- one center tile plus its 6
neighbors -- using wgpu and `tiles_tools::coordinates::hexagonal`'s axial
coordinate system to place each instance. It shows how to turn hexagonal grid
coordinates into world-space instance transforms and draw them natively
through wgpu.

This example provides a foundation for hex-grid game boards, tile-based maps,
or other applications built on `tiles_tools`' hexagonal coordinate types.

![image](showcase.webp)

**[How to run](../../how_to_run.md)**
