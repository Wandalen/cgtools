# Invariant: Rhai-Facing Names Mirror Rust Identifiers

Every constructor and type name registered into the Rhai engine textually matches the Rust identifier it wraps — never a generic alias.

### Scope

- **Purpose**: Pin the naming convention linking every script-visible name to its Rust identifier, and be explicit that it is convention-enforced, not tooling-enforced.
- **Responsibility**: State the rule precisely and record the honest state of its enforcement.
- **In Scope**: Type names and constructor function names registered via `register_type_with_name`/`register_fn`.
- **Out of Scope**: Method and operator names, which already follow Rhai's own operator-overload spelling (`+`, `-`, `*`) or the corresponding Rust method name directly (`update`, `value`, `is_completed`) — these are not a separate naming choice this invariant needs to additionally pin.

### Invariant Statement

For every type and constructor registered into the engine, the Rhai-visible name textually matches the Rust identifier it wraps: unchanged for a type name (`F32x2` → `"F32x2"`, `F64x2` → `"F64x2"`), lowercased for a constructor function name (`F32x2` → `"f32x2"`, `F64x2` → `"f64x2"`, `Tween` → `"tween"`).

### Enforcement Mechanism

**Manual code review only — no automated check exists.** This is stated plainly rather than aspirationally: nothing in the test suite, in `top_level_lint.rs`, or anywhere else would catch a future registration using a generic alias (e.g. registering `F32x2` as `"Vec2"` instead of `"F32x2"`). This rule is promoted here from the crate [`readme.md`](../../readme.md)'s own "Naming convention" prose specifically to make that enforcement gap visible as a stated fact, rather than leaving it implicit and easy to assume is checked somewhere.

### Violation Consequences

A script author who reads Rust-side documentation (rustdoc, or this doc set) and expects the same name to work from a script would find a mismatch if this rule were ever violated — breaking the "read the Rust type name, use the same name in script" mental model this crate's binding surface is designed around, and undermining discoverability without any compiler or test signal pointing at the cause.

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/002_dual_precision_side_by_side_registration.md](../pattern/002_dual_precision_side_by_side_registration.md) | The pattern whose extensibility depends on this naming rule holding for every future addition |

### Sources

| File | Relationship |
|------|--------------|
| `src/vector_binding.rs` | `f32x1_register`, `f32x2_register`, `f32x3_register`, `f32x4_register`, `f64x1_register`, `f64x2_register`, `f64x3_register`, `f64x4_register` — every name registered here follows this rule |
| `src/tween_binding.rs` | `tween_f32x1_register`, `tween_f32x2_register`, `tween_f32x3_register`, `tween_f32x4_register`, `tween_f64x1_register`, `tween_f64x2_register`, `tween_f64x3_register`, `tween_f64x4_register` — same |
| `readme.md` | § Naming convention — the informal prose source of this rule, now also pinned as a checked-in-principle invariant |

### Tests

No dedicated regression test pins this — as the Enforcement Mechanism section states, nothing currently checks it automatically. A future test could assert the registered name set against a derived list of Rust identifiers if this gap is ever worth closing.
