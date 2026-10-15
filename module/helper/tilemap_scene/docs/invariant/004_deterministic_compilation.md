# Invariant: Deterministic Compilation

Same spec, same scene, same time, same seed — same command stream, every
run. Variation is opt-in via the scene's own seed, never ambient.

### Scope

- **Purpose**: Pin that scene compilation is a pure function of its inputs — the tile stack's determinism invariant, and the property snapshot testing stands on.
- **Responsibility**: State the property, enumerate the design choices enforcing it, and record what breaks when it fails.
- **In Scope**: The `compile` layer's output as a function of `(spec, scene, time, seed)`.
- **Out of Scope**: Determinism of *rendering* the commands (a backend concern); cross-platform floating-point identity of every derived value (the contract is per-build run-to-run identity; lattice math is integer-based).

### Invariant Statement

Compiling the same `RenderSpec` and `Scene` at the same time value yields an
identical command stream on every run. All apparent randomness (state
variation, jitter) is pseudo-random, derived from the scene's own 64-bit
`seed` — no wall clock, no runtime RNG, no OS entropy, no
iteration-order dependence reaches the output.

### Enforcement Mechanism

- **Seeded pseudo-randomness**: `Scene` carries an explicit 64-bit `seed`
  (`src/scene.rs`); variation is computed by seeded hashing, salted per use
  site — `src/compile/frame.rs` documents its selection as "Deterministic
  pseudo-random — seeded from `Scene.seed`", and `src/source.rs` states the
  design intent: variation is not "'random' in the runtime-RNG sense".
- **No ambient inputs**: the compile layer takes time as an explicit
  parameter and has no dependency able to read a clock or entropy source
  (see [003_compiles_to_renderer_commands_only.md](003_compiles_to_renderer_commands_only.md)'s dependency surface).
- **Order discipline**: hash maps (`rustc_hash::FxHashMap` — a fixed,
  unseeded hasher) are used for keyed lookup, while output ordering is
  imposed explicitly — draw lists pass through `apply_sort_mode`
  (`src/compile/frame.rs`) rather than inheriting map iteration order.
- **Snapshot infrastructure as the watchdog**: the `hash` and `snapshot`
  layers exist to fingerprint compiled output; any nondeterminism surfaces
  as a failing snapshot comparison rather than a silent flake.

### Violation Consequences

- Snapshot/golden tests flake — the crate's own primary test strategy stops
  working.
- Replay, seeking, and off-screen re-rendering desynchronize: the same scene
  file renders differently between a live run and an export.
- A scene's look stops being an artifact of the scene *file* — undermining
  the scene-as-script contract that the file fully determines the picture.

### Invariants

| File | Relationship |
|------|--------------|
| [003_compiles_to_renderer_commands_only.md](003_compiles_to_renderer_commands_only.md) | Supplies the dependency-surface half of the enforcement (nothing able to read clocks or entropy) |

### Sources

| File | Relationship |
|------|--------------|
| `src/compile/frame.rs` | Seeded, salted pseudo-random selection; explicit `apply_sort_mode` output ordering |
| `src/scene.rs` | The 64-bit per-scene `seed` field |
| `src/snapshot.rs` | "Per-scene seed for deterministic pseudo-randomness" — snapshot fingerprinting built on this invariant |
| `src/source.rs` | Design statement that state selection is not runtime-random |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_compile_test.rs` | Dedicated determinism tests: `variant_hashcoord_picks_deterministically`, `variant_random_deterministic_across_frames`, `palette_expansion_produces_same_tiles_as_explicit` |
