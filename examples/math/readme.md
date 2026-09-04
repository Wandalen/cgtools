# 🔬 Math Examples

Native demos of the math crates (`ndarray_cg` and friends) — computation first, no GPU API required.

## 🚀 How to Run

Each example is a native binary — no wasm target or trunk needed:

```bash
cd <example>
cargo run --release --all-features
```

Or, from any directory, by partial unique match against the example path:

```bash
action/run life
```

## 📂 Examples

| Example | Responsibility |
|---------|----------------|
| `life/` | Conway's Game of Life on `ndarray_cg` grids with efficient neighbor computation |
