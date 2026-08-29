# Invariant: Zero Dependencies

`cargo tree -p deterministic_math` prints one line. The crate has no
dependencies, no dev-dependencies, and no optional dependencies behind a
feature — and that is a load-bearing property, not a side effect of the crate
being small.

### Scope

- **Purpose**: Record that the dependency count is zero by design, what it buys, and the two workspace conventions this crate is exempt from in order to keep it.
- **Responsibility**: State the property, its rationale, its enforcement, and its cost.
- **In Scope**: `Cargo.toml`'s `[dependencies]`, `[dev-dependencies]`, and `[features]` tables.
- **Out of Scope**: The libm ban, which is about calls rather than crates (see [002](002_pinned_operations_only.md)) — though the two protect the same guarantee from opposite directions.

### Invariant Statement

`[dependencies]` and `[dev-dependencies]` are empty. `[features]` declares
`enabled` / `default` / `full` for uniformity with the rest of the workspace —
feature unification refers to those names — and every one of them is empty by
construction, enabling nothing.

### Why It Is Load-Bearing

**A dependency is a libm the scan cannot see.**
[002](002_pinned_operations_only.md) scans this crate's own sources. It has no
reach into a dependency's, so a transitive crate calling `f64::sin` in a helper
would void [001](001_bit_reproducibility.md) with nothing failing. Zero
dependencies is how that stays structurally impossible rather than merely
unlikely.

**The consumers who want this have no use for a graphics stack.** Reproducible
arithmetic is wanted by simulation code, replay harnesses and lockstep
netcode — none of which want a rendering workspace's transitive closure to get
it. This crate was extracted from `ndarray_cg` for exactly this reason: the
functions were correct where they were, but reaching them cost a linear-algebra
crate.

**`mod_interface` was the specific thing given up.** The workspace's standard
module-composition macro carries a transitive closure of 68 crate-versions
(57 distinct crate names) — count it yourself with the recipe under Example
below. Adopting it here would trade the crate's entire reason for existing for
a namespacing convenience. `src/lib.rs` therefore uses plain `pub mod` and
`pub use` instead — 42 of the workspace's 49 crates use `mod_interface`, and
this is one of the seven that do not.

No crate-local `rulebook.md` records that, deliberately. The workspace rulebook
does not *mandate* `mod_interface` — it acknowledges the repository uses the
macro but sets no rule requiring it — so there is no workspace rule here to
override, and its documentation-layout section says a crate-local `rulebook.md`
exists "only when overrides are needed; absent by default." Writing one to
record a departure from a convention that was never a rule would make this the
only crate in 49 carrying a rulebook, for a rule that does not exist. The
departure is recorded here instead, which is where a reader looking for the
crate's dependency contract will actually be.

`error_tools` is absent for a simpler reason: nothing here returns a `Result`.
Every function's failure mode is a value in the return type — `NaN`, `±INFINITY`
or a saturating bound — because that is what a numeric kernel's callers can act
on without a branch per call.

### Enforcement Mechanism

The `[dependencies]` and `[dev-dependencies]` tables are empty in `Cargo.toml`,
with a comment stating why. There is no automated assertion — the property is
visible in one command:

```sh
cargo tree -p deterministic_math
```

A guard test comparing the manifest against an expected string was considered
and rejected: it would assert the file's text rather than the resolved graph,
so it would pass while a workspace-level `[patch]` or a feature-unified
transitive edge added something. `cargo tree` reads the graph that is actually
built, which is the thing the invariant is about.

Run it before and after any change that touches the manifest.

### Violation Consequences

Adding a dependency degrades the guarantee in two independent ways at once. It
introduces source this crate's libm scan cannot inspect, so
[001](001_bit_reproducibility.md) becomes a claim about only part of what ships.
And it puts a transitive closure between a simulation crate and the reproducible
arithmetic it needs, which is the cost the extraction was performed to remove.

Neither shows up as a test failure. Both show up as the crate no longer being
worth choosing over the platform's libm.

### Example

```sh
$ cargo tree -p deterministic_math
deterministic_math v0.1.0 (/…/module/math/deterministic_math)
```

One line, no children. Compare against the macro this crate deliberately does
not use — `--prefix none | sort -u` collapses the tree to the set of distinct
crate-versions actually pulled in, which is the number that matters:

```sh
$ cargo tree -p deterministic_math --prefix none | sort -u | wc -l
1
$ cargo tree -p mod_interface --prefix none | sort -u | wc -l
68
```

Both figures are a snapshot of the current lockfile; the ratio is the durable
part.

### Invariants

| File | Relationship |
|------|--------------|
| [001_bit_reproducibility.md](001_bit_reproducibility.md) | The guarantee a dependency's own unscanned libm call would silently void |
| [002_pinned_operations_only.md](002_pinned_operations_only.md) | The same protection inside this crate's boundary; this invariant covers what that scan cannot reach |

### Sources

| File | Relationship |
|------|--------------|
| `Cargo.toml` | The empty dependency tables and the comment recording why they are empty |
| `../../../rulebook.md` | The workspace rulebook — which acknowledges `mod_interface` without mandating it, and sets crate-local rulebooks as absent by default |
| `src/lib.rs` | Plain `pub mod` / `pub use` composition, in place of `mod_interface!` |

### Tests

None, deliberately — see Enforcement Mechanism. `cargo tree` inspects the
resolved dependency graph; a test could only inspect the manifest text, which is
a weaker statement than the invariant makes.
