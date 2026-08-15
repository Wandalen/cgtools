# voronoi

Cellular (Worley) F1 distance and cell id at a 2D point. Each unit lattice
cell holds one [`hash22`](../hash22/readme.md)-jittered feature point; the
function returns the distance to the nearest one plus a stable per-cell
identifier — organic cells, cracked surfaces, starfields with guaranteed
minimum spacing.

## Visualization

![voronoi preview](preview.png)

Rendered via the chunk-preview harness's synthesized field: `voronoi( p )`
is evaluated directly — no wrapper needed, its native `vec2f → vec2f`
shape matches the harness's Vec2 mode — with its two output channels
(`.x` = F1 distance, as in the original grayscale preview) mapped to
red/green (blue held at a fixed `0.5` pad), at `preview_scale = 8`.
Directly previewable via `sch preview voronoi`.

## Parameters

| Field | Value |
|---|---|
| `name` | `voronoi` |
| `description` | Cellular (Worley) F1 distance and cell id at a 2D point. |
| `tags` | `category:noise`, `technique:cellular` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | `hash22` |
| `export` | `fn voronoi(p: vec2f) -> vec2f` |

## Nuances

- The 3×3 neighborhood scan is exact, not approximate: jitter stays inside
  each unit cell (`hash22` is `[0, 1)`), so the nearest feature point can
  never live farther than one cell away.
- Squared distances are compared in the loop; the single `sqrt` happens
  once at the end — the returned `.x` is true euclidean distance.
- `.y` is the winning cell's `jitter.x` — constant across the whole cell,
  so it works as a per-cell random seed (color, brightness, phase). It is
  an id, not a spatial quantity.
- F1 near cell corners can slightly exceed 1 in `p` units; treat the range
  as ~`[0, 1.2]` when normalizing for display.

## Relatives

- **Depends on:** [`hash22`](../hash22/readme.md) (per-cell feature-point
  jitter and the id channel).
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get voronoi`, `sch tree voronoi`)
- **Consumers:** none yet.
