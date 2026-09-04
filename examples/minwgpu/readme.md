# 🔬 Native wgpu Examples with `minwgpu`

Examples targeting native GPU rendering through [wgpu](https://wgpu.rs) — desktop windows, not browsers.

## 🚀 How to Run

Each example is a native binary — no wasm target or trunk needed:

```bash
cd <example>
cargo run --release --all-features
```

Or, from any directory, by partial unique match against the example path:

```bash
action/run hello_triangle
```

## 📂 Examples

| Example | Responsibility |
|---------|----------------|
| `flecs_bouncing_circles/` | 2D physics simulation driven by the flecs ECS, rendered with wgpu |
| `grid_render/` | Efficient grid-pattern rendering in a native wgpu window |
| `hello_triangle/` | Classic "Hello Triangle" — the minimum code for native wgpu rendering |
