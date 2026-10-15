# Pitfall: Hexagonal `Axial` Coordinate Has Two `distance` Methods

### Scope

- **Purpose**: Warn that `Coordinate<Axial, Orientation>` carries two same-named `distance` methods that resolve differently depending on whether the argument is passed by value or by reference.
- **Responsibility**: Document the exact call-shape rule Rust uses to pick between them, and why generic code never sees the inherent one.
- **In Scope**: `hexagonal::Coordinate<Axial, Orientation>`'s inherent `distance` method and its `Distance` trait impl.
- **Out of Scope**: The distance formula itself, which is identical between the two (see `algorithm/001`).

### Trap

Calling `coord.distance(other)` on two owned `hexagonal::Coordinate<Axial, Orientation>` values (no `&`), inside code that also uses generic helpers written against `C: Distance`, and assuming both call sites compute "the same `distance` method."

### Failure

Two methods named `distance` coexist on the same type, with different signatures (full comparison: `algorithm/001`):

- Inherent method (`src/coordinates/hexagonal.rs:156-164`): `fn distance(&self, other: Self) -> i32` — takes `other` **by value**, returns **signed `i32`**.
- `Distance` trait impl (`src/coordinates/hexagonal.rs:390-402`): `fn distance(&self, other: &Self) -> u32` — takes `other` **by reference**, returns **unsigned `u32`**.

Rust's method resolution picks whichever is callable for the argument shape given: `coord.distance(other)` (owned) silently resolves to the inherent `i32`-returning method; `coord.distance(&other)` resolves to the trait's `u32`-returning method. No compiler warning fires either way — both are valid, unambiguous calls once the argument shape is fixed. Any code written against a generic `C: Distance` bound (e.g. `algorithm/002`'s A* implementation) only ever sees the trait method — the inherent method is reachable solely through concrete, non-generic `Coordinate<Axial, Orientation>` method syntax. A hand-written concrete call site that happens to pass an owned value therefore computes a different-typed result (`i32`) than what any generic helper computes for the identical pair of coordinates (`u32`), with nothing in the call syntax flagging the divergence.

### Mitigation

Always call with an explicit reference — `coord.distance(&other)` — to deterministically select the trait method, matching what all generic (`C: Distance`) code sees. Reserve the by-value inherent call only when the signed `i32` return is specifically and intentionally wanted.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_coordinate_distance_and_neighbor_formulas.md](../algorithm/001_coordinate_distance_and_neighbor_formulas.md) | Full side-by-side comparison of both methods' signatures and arithmetic |

### Sources

| File | Relationship |
|------|--------------|
| `src/coordinates/hexagonal.rs` | Both `distance` methods on `Coordinate<Axial, Orientation>` |

### Tests

No test currently exercises both methods on the same input pair and asserts their numeric results agree — see `algorithm/001`'s Tests section for what coverage does exist.
