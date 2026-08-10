# Pattern: Script-as-Glue

The imperative form of the L5 scene-script contract: the script is a
program in an embedded language, executed against engine vocabulary that
bindings deliberately expose. Behavior lives in the script; the host
provides the verbs.

### Scope

- **Purpose**: Name and pin the script-form under which expressiveness is the construction guarantee and determinism is an authorial discipline.
- **Responsibility**: Define the pattern's guarantees, its costs, and its known use for 2D scenes.
- **In Scope**: The pattern itself and the criteria for choosing it.
- **Out of Scope**: The declarative alternative (see [004_script_as_data.md](004_script_as_data.md)); the L5 layer contract both forms serve (see [../layer/006_l5_scene_script_and_runners.md](../layer/006_l5_scene_script_and_runners.md)).

### Problem

Some scene work is open-ended by nature — response to input, ad hoc motion
logic, live iteration during authoring — and a fixed document schema is a
straitjacket for it. Every new behavior would mean a host-compiler change
and a redeploy, exactly the loop scripting exists to break.

### Solution

Embed a scripting language and curate its surface:

- The host assembles an interpreter (`build_engine()` in the known use,
  over Rhai) and registers bindings — each binding module exposes one slice
  of engine vocabulary (vectors, tweens) into script space.
- The script is code: it calls the bound API imperatively, and new behavior
  is written in script without touching the host.
- The binding surface is the contract: what is not bound does not exist for
  scripts, which is how the host keeps the exposure deliberate.

### Consequences

- **Unbounded expressiveness within the bound surface**: behavior is
  limited by the bindings, not by a schema.
- **Live iteration**: scripts reload without recompiling the host — the
  authoring loop this pattern exists for.
- **Determinism by discipline** (the cost): nothing structural stops a
  script from time-dependent or order-dependent behavior —
  `top_level_lint` checks *where* imperative code may live (inside
  `main()`, never at top level), not *whether* it behaves deterministically,
  pinned only for shape by
  [`scene_script` invariant/001](../../module/helper/scene_script/docs/invariant/001_top_level_bindings_convention.md),
  never for full determinism. L5's "same script → same frames" contract
  must still be kept by the author and can only be spot-checked, never
  proven, from outside.
- **Scripts are opaque to tools**: not validatable without execution, not
  diffable as structured data — a script diff is a code diff.
- **Every binding is an API commitment**: the exposed surface must be
  curated and versioned like any public API.
- **Declarative shape does not imply script-as-data**: `top_level_lint`
  (`scene_script`) enforces top-level *shape* only — no loop, branch, or
  mutation sitting bare at top level — never whether the script calls into
  engine-registered vocabulary. A script can be shape-declarative and still
  be script-as-glue in substance. Concrete boundary case, verified directly
  against the source:
  `examples/scene_script/f32x2_vector_arithmetic/src/f32x2_vector_arithmetic.rhai`
  is a `let`/`let`/trailing-expression sequence — structurally declarative,
  accepted by the checker — yet it calls the registered `f32x2(...)`
  constructor and the registered `+`/`*` operator overloads
  (`vector_binding.rs`'s `register_fn` calls): genuine engine calls, which
  makes it script-as-glue by this pattern's own defining property (a
  script-as-data document "cannot call the engine" — see
  [004_script_as_data.md](004_script_as_data.md)) regardless of its
  declarative-looking shape. Never infer pattern membership from
  `top_level_lint` passing, or from a script's mere absence of loops and
  branches — check whether it calls a registered binding instead.

### When to Choose

Interactive tooling and live-authoring loops, or when the script author is
the engine consumer. For a new stack's L5, reach for
[script-as-data](004_script_as_data.md) first and add glue only where data
cannot express the need — the determinism contract is the harder thing to
retrofit.

### Patterns

| File | Relationship |
|------|--------------|
| [004_script_as_data.md](004_script_as_data.md) | The contrasting form: trades this pattern's expressiveness for structural determinism |

### Layers

| File | Relationship |
|------|--------------|
| [../layer/006_l5_scene_script_and_runners.md](../layer/006_l5_scene_script_and_runners.md) | The layer contract this pattern is one realization of |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/scene_script/src/engine.rs` | Interpreter assembly (`build_engine()`) |
| `module/helper/scene_script/src/tween_binding.rs` | Binding slice: animation tweens |
| `module/helper/scene_script/src/vector_binding.rs` | Binding slice: vector math |
| `module/helper/scene_script/src/top_level_lint.rs` | Structural check: imperative code confined to `main()` |
