# Pitfall: Script Functions Can't See Outer-Scope Bindings

### Scope

- **Purpose**: Warn a script author that a Rhai `fn` body sees only its own parameters and locals — never a top-level `let`/`const`, even though both live in the same file and the function is defined right below the binding.
- **Responsibility**: Document the concrete failure mode and the established mitigation (re-seed state as parameters).
- **In Scope**: Rhai's function-scoping rule as it interacts with the top-level bindings convention (`invariant/001`), which pushes all mutable state *into* `main()`.
- **Out of Scope**: Rhai's closure syntax (`|...|`), which *can* capture outer scope explicitly — not used anywhere in this crate's bindings or examples.

### Trap

It reads naturally to declare shared setup once at top level and expect
every `fn` in the same script to see it — that's how a top-level `let` in
most scripting languages with lexical scoping behaves. Rhai does not work
this way: a script function's body is scoped to its own parameters and its
own locals only. A top-level `let`/`const` binding is invisible from inside
any `fn`, regardless of definition order or how "global" it looks in the
source text.

This collides directly with the top-level bindings convention
([`invariant/001`](../invariant/001_top_level_bindings_convention.md)):
that convention pushes *all* imperative code — including state mutation —
inside `main()`, which means any state the simulation mutates over time
must already be reachable from inside `main()`. A top-level `let` alone
does not achieve that.

### Failure

Given `let ball_pos = f32x2( 100.0, 50.0 ); fn main() { ball_pos.x += 1.0; }`,
compilation succeeds — Rhai does not statically reject the reference at
parse time — but calling `main()` fails at runtime with `Variable not
found: ball_pos`, confirmed directly against the engine. There is no
silent fallback to a fresh, independently-scoped `ball_pos`: the name is
simply unresolved inside `main()`'s own scope, one level removed from the
"looks shared, isn't" framing a same-named top-level `let` invites.

### Mitigation

Pass every piece of state `main()` needs to mutate as an explicit
parameter, and re-`let` a fresh local from it on entry — this is exactly
`examples/scene_script/pingpong_animation/src/pingpong_animation.rhai`'s
structure: `ball_start_pos`, `ball_start_vel`, `paddle_start_y` are
top-level `let` bindings (read-only seed values, never mutated at top
level — legal under `invariant/001`), passed into
`main( court, paddle_speed, ball_start_pos, ball_start_vel, paddle_start_y, ticks )`,
which immediately re-binds working locals (`let ball_pos = ball_start_pos;`,
etc.) before the loop mutates them. The top-level names read as the
simulation's *initial* values; the `main()`-local names are the *live*
ones — never conflate the two, and never expect a `main()` reference to a
top-level name to see mutations made inside `main()` reflected back at top
level (it can't — top level has already finished executing by the time
`main()` runs, and `main()`'s own copy is independent from the start).

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_top_level_bindings_convention.md](../invariant/001_top_level_bindings_convention.md) | The convention that pushes mutable state into `main()`, where this scoping rule then applies |

### Sources

| File | Relationship |
|------|--------------|
| `examples/scene_script/pingpong_animation/src/pingpong_animation.rhai` | Worked example: top-level seed values re-bound as fresh `main()` locals |

### Tests

No dedicated regression test pins this — it's a fact about the Rhai
language runtime, not a claim this crate implements or could regress on.
`example_scripts_follow_declarative_top_level_convention`
(`tests/example_convention_test.rs`) indirectly relies on the mitigation
pattern being followed correctly (`pingpong_animation.rhai` compiles and
runs to produce its expected frame output), but does not test the
scoping rule itself.
