# voronoi

Cellular (Worley) F1 distance and cell id at a 2D point. Each unit lattice
cell holds one [`hash22`](../hash22/readme.md)-jittered feature point; the
function returns the distance to the nearest one plus a stable per-cell
identifier — organic cells, cracked surfaces, starfields with guaranteed
minimum spacing.

## Visualization

![voronoi preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field:
`voronoi_preview( p, 1.0, 1.0, 0.0 ) = voronoi( p, 1.0, 1.0, 0.0 ).x` —
written straight to `vec3f( value )`, clamped to `[0, 1]`, at
`preview_scale = 8`. Drag `jitter` to `0` for a perfectly regular grid of
feature points, past `1` for a more exaggerated/chaotic look, or leave it at
`1` for this chunk's original, fully-randomized look. Drag `metric` across
`0` through `4` to sweep cell shape from diamond (manhattan) through round
(euclidean — the default, at the midpoint) to square (chebyshev), with two
intermediate softened shapes along the way. Drag `seed` away from `0`
to get a different arrangement of feature points entirely, same `jitter`/
`metric`. Directly previewable via `sch preview voronoi`.

## Parameters

| Field | Value |
|---|---|
| `name` | `voronoi` |
| `description` | Cellular (Worley) F1 distance and cell id at a 2D point. |
| `tags` | `category:noise`, `technique:cellular` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | `hash22` |
| `export` | `fn voronoi(p: vec2f, jitter: f32, metric: f32, seed: f32) -> vec2f`, `fn voronoi_preview(p: vec2f, jitter: f32, metric: f32, seed: f32) -> f32` |

## Nuances

- `jitter` (`//@ param:`, range `[0, 2]`) scales the per-cell `hash22`
  offset: `0` collapses every feature point onto its own grid corner (a
  perfectly regular lattice); `1` (this range's midpoint) reproduces this
  chunk's original, fully-randomized behavior; above `1` pushes feature
  points past their own cell, for a more exaggerated/chaotic look. Values
  above `1` are a genuinely new capability, not just an exposed constant —
  see the search-radius bullet below for how correctness is preserved.
- `metric` (`//@ param:`, range `[0, 4]`) rounds to a discrete selector
  spanning the Minkowski Lp family — `dist = (|dx|^p + |dy|^p)^(1/p)` —
  ordered by increasing `p`: `0` = manhattan (`p=1`, `|dx| + |dy|`, diamond
  cells), `1` = `p=1.5` (softened diamond), `2` = euclidean (`p=2`,
  `sqrt(dx² + dy²)`, round cells — this chunk's original behavior), `3` =
  `p=4` (softened square), `4` = chebyshev (`p=∞`, `max(|dx|, |dy|)`, square
  cells — the Lp limit, computed directly since `pow()` can't reach
  infinity). It's declared `f32`, not a true enum — this codebase's
  slider/uniform pipeline is `f32`-only for `argument`-kind parameters, so
  the shader rounds internally. Euclidean is deliberately placed at the
  range's midpoint (`2`) precisely because a slider's default value *is*
  its range's midpoint (see `shader_chunks_preview_core`'s `slider_of`) —
  any other assignment would make a freshly opened preview default to a
  different cell shape than this chunk's classic look.
- `seed` (`//@ param:`, range `[-50, 50]`) offsets the integer lattice
  coordinate fed into `hash22`, reshuffling which feature point lands in
  which cell. `0` (this range's midpoint) reproduces the original, unseeded
  pattern. This is deliberately different from panning `p` itself: panning
  by an integer amount just relabels the same cell → hash mapping across
  cells (the pattern looks identical, only shifted), while offsetting the
  coordinate that's actually hashed genuinely decorrelates it, since
  `hash22` has no smoothness to preserve between neighboring inputs.
- The neighbor search radius grows with `jitter` — `ceil(jitter)`, not a
  fixed `3×3` — because past unit jitter, a cell `k` steps away can reach
  into the query's own cell once `jitter >= k` (that cell's nearest
  reachable point is `k - jitter` away), so a fixed window would silently
  miss the true nearest point above `jitter = 1`. `ceil(jitter)` is the
  smallest radius that's always exact, and holds across every `metric`
  value above: each one is `>= ` chebyshev distance for the same offset, so
  bounding the (metric-agnostic) per-axis reach bounds them all. Reduces to
  the original fixed radius `1` exactly at `jitter = 1`.
- The per-candidate sentinel is derived from the query's own cell — always
  in-window — whose offset from the query point is strictly less than
  `max(1, jitter)` per axis (hash outputs are in `[0, 1)`), times a small
  margin against floating-point edge cases at that boundary. Sum-of-powers
  metrics (manhattan through `p=4`) double it, since both axes could hit
  their bound at once; chebyshev takes the single larger axis, unbounded by
  a second term — so the exact formula depends on which `metric` is active,
  not one constant shared across all five.
- Search cost scales with the radius: `(2 * ceil(jitter) + 1)²` candidates
  per lookup — `9` at the default `jitter = 1` (unchanged from before this
  chunk had a `jitter` parameter at all), up to `25` at `jitter = 2`. Still
  cheap for a single fragment shader, but non-constant, unlike `metric`/
  `seed` which don't affect iteration count.
- Every metric except chebyshev accumulates a cheaper, monotonic surrogate
  per candidate — the Lp sum `|dx|^p + |dy|^p`, not yet raised to `1/p` —
  and roots it once at the end instead of once per candidate; chebyshev's
  `max()` is already a true distance and is never rooted. The returned
  `.x` is always a true (non-surrogate) distance, regardless of `metric`.
- `.y` is the winning cell's `rnd.x`, the raw (un-jittered) `hash22`
  x-channel — constant across the whole cell, so it works as a per-cell
  random seed (color, brightness, phase) independent of `jitter`/`metric`/
  `seed`. It is an id, not a spatial quantity.
- F1 near cell corners can slightly exceed 1 in `p` units at the default
  `jitter = 1`; treat the range as ~`[0, 1.2]` when normalizing for display
  at that setting. That figure is calibrated for euclidean at `jitter = 1`
  — for the same points, distance trends higher moving toward manhattan
  and lower moving toward chebyshev (the Lp family is monotonically
  decreasing in `p`: L1 ≥ L1.5 ≥ L2 ≥ L4 ≥ L∞ always), and the whole range
  scales up further as `jitter` increases past `1`.

## Relatives

- **Depends on:** [`hash22`](../hash22/readme.md) (per-cell feature-point
  jitter and the id channel).
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get voronoi`, `sch tree voronoi`)
- **Consumers:** none yet.
