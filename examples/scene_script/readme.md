# 🔬 Scene Script Examples

Native demos driving scenes from `.rhai` scripts — the logic lives in the script, the host only executes it. See [script-as-glue](../../docs/pattern/005_script_as_glue.md) for the pattern these examples demonstrate.

## 🚀 How to Run

Each example is a native binary — no wasm target or trunk needed:

```bash
cd <example>
cargo run --release --all-features
```

Or, from any directory, by partial unique match against the example path:

```bash
action/run pingpong_animation
```

## 📂 Examples

| Example | Responsibility |
|---------|----------------|
| `f32x2_vector_arithmetic/` | Builds and combines `F32x2` vectors with `+`/`*` operators inside a `.rhai` script |
| `pingpong_animation/` | Pong-style scene scripted entirely in `.rhai` — loops, branches, vector arithmetic |
