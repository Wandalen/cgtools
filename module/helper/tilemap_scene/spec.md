# Tilemap Renderer Scene Model — Specification

**Version:** 0.2.0
**Status:** draft — breaking rewrite from 0.1.0
**Target:** reference implementation in Rust using `serde` + `ron`.
**Scope:** the file format and the rendering algorithm for 2D tile-based games. Does NOT cover: asset pipeline, game logic, pathfinding, networking, UI toolkit. Renderer integration API (`set_animation`, `spawn_object`, etc.) is listed in §14 as a contract, not fully specified here.

## 0. What this format describes

A scene describes **how to render 2D tile-based games from sprite stacks**. It is designed to cover, with one compositional vocabulary, games in the style of:

- Battle for Wesnoth (hex tactical wargame with rich terrain transitions).
- Slay the Spire / HoMM overworld (hex map with decorative objects).
- Stardew-, Rimworld-, Factorio-style square tilemaps (deferred to a later revision but not excluded by the model).

The model is intentionally minimal: the renderer is not told "these are terrains", "these are units". It is given **objects made of layered textures with per-layer behaviour**, and it draws them. Game-specific classification (terrain / unit / overlay / prop) lives in user data, not in the format vocabulary.

## 1. Core Model

Three concepts. Everything else is built from them.

### 1.1 Object

An **Object** is the atomic unit of a scene. It has:

- **anchor** — how the object is attached to the world (see §3).
- **global_layer** — the name of the pipeline z-bucket the object draws into (see §8).
- **states** — a map of named layer stacks. Names are user-defined (`"idle"`, `"walk"`, `"default"`, `"dusk"`). At most one state is active per object instance at a time. Each state's layer stack may use any `SpriteSource` (`Static`, `Animation`, composite neighbour-aware, …), so a "state" covers both genuine character moods *and* simple one-layer-one-sprite setups.
- **default_state** — the state active when the game has not issued a `set_state` call.

The renderer does not know *why* a state is active. Game logic external to the renderer calls `set_state(object_instance, name)` at will.

### 1.2 Layer

A **Layer** is one textured strip within an object. An object is an ordered stack of layers drawn bottom to top.

A layer has:

- **sprite_source** — a rule that chooses one sprite/frame at render time (§5).
- **behaviour** — what the renderer does to that sprite: tint, mask, blend mode, effects, parallax, alpha, tiling (§6).
- **z_in_object** — ordering within the object's layer stack.
- **pipeline_layer** *(optional)* — override for which pipeline bucket this layer draws into. If absent, the layer inherits the object's `global_layer`. Use this to place edge-transition or overlay layers of one object into a later pipeline pass than the object's base (see §5.5 Wesnoth edge idiom).

**Invariant:** `sprite_source` and `behaviour` are orthogonal. A frame-animated sprite with a static flat tint, a static sprite with a frame-animated mask, and a frame-animated sprite with a frame-animated mask are all valid combinations declared the same way.

### 1.3 Anchor

An **Anchor** declares how an object is placed in the world. It determines:

- what "position" means for an instance (one grid coord, a pair, a triple, a pixel point, or the viewport);
- what "neighbours" and "context" are visible to the object's sprite sources;
- how the object is culled and sorted.

Anchors are enumerated in §3.

## 2. Coordinate Systems and Tiling Strategy

### 2.1 Tiling strategy

A scene picks one **tiling strategy** for its grid:

```ron
tiling: HexFlatTop | HexPointyTop | Square4 | Square8
```

The strategy determines:

- Number of neighbours per cell (6 for hex, 4 or 8 for square).
- Neighbour direction enumeration order (used for `NeighborBitmask` bit indices).
- Dual-mesh geometry: 3 corners per vertex for hex, 4 for square.
- Pixel conversion from grid coordinates to screen coordinates.
- Canonical `Multihex.shape` offset conventions.

**Version 0.2.0 implements `HexFlatTop` and `HexPointyTop` only.** `Square4` and `Square8` are reserved values and MUST be rejected at load time with a clear error. The model is designed so that a future minor version can implement them without breaking the format.

### 2.2 Hex coordinates

Axial coordinates `(q, r)` with `i32` components. Cube coordinates are derived as `(q, r, -q - r)` where needed.

### 2.3 Direction ordering

Directions are enumerated clockwise starting from the top-most. Bit `i` of a neighbour bitmask corresponds to the neighbour at index `i`.

```
HexFlatTop   : [N, NE, SE, S, SW, NW]
HexPointyTop : [NE, E, SE, SW, W, NW]
```

### 2.4 Pixel conversion

Given hex size `(w, h)` (full bounding box in pixels):

```
HexFlatTop:
    x = q * w * 0.75
    y = r * h + (q & 1) * h * 0.5

HexPointyTop:
    x = q * w + (r & 1) * w * 0.5
    y = r * h * 0.75
```

Screen coordinates assume y-down.

## 3. Anchors

### 3.1 `Hex`

Object occupies one cell. Position = one `(q, r)`. Neighbours = 6 (or 4/8 for square in future revisions).

Used for: terrain, units, overlay objects (villages, forests), hex-anchored connected objects (walls as hex fills).

### 3.2 `Edge`

Object lives on the edge between two cells. Position = `(hex, direction)`: the hex plus the side to its neighbour.

Used for: fences between cells, rivers as linear objects, roads drawn along edges.

**Canonicalization:** an edge `(hex_A, dir_AB)` and `(hex_B, dir_BA)` is the same edge. The canonical form is the `(hex, direction)` pair whose hex has the lexicographically smaller `(q, r)`. Renderers MUST emit each edge exactly once.

**Sprite sources applicable to `Edge`:** `Static`, `Variant`, `Animation`, `External`, `EdgeConnectedBitmask` (§5.9). Hex-specific sources (`NeighborBitmask`, `NeighborCondition`, `VertexCorners`) are rejected at load time.

### 3.3 `Vertex`

Object lives on a grid vertex where multiple cells meet (3 for hex, 4 for square). Position = tuple of cells sharing the vertex.

Used for: dual-mesh triangle blends.

**Canonicalization:** the tuple of corner cells is sorted lexicographically by the `terrain-id-at-that-cell` used during matching, and a `rotation` integer records the permutation applied. For hex, rotation ∈ {0, 1, 2}. Renderers MUST emit each vertex exactly once.

### 3.4 `Multihex`

Object occupies multiple cells. Position = anchor cell + `shape` (list of relative offsets in grid coordinates).

```ron
anchor: Multihex(shape: [(0, 0), (1, 0), (0, 1), (1, 1)])  // 2x2 castle
```

Rendering:

- Pixel position is the pixel position of the anchor cell.
- A single sprite covers the shape's bounding box.
- Culling: the object is drawn if **any** cell in its shape is visible.
- Y-sort: by the anchor cell's Y unless `sort_y: BottomOfShape` is specified on the object (optional override).

**Restrictions in 0.2.0:** Multihex objects MUST use `sprite_source: Static` or `Animation`. Neighbour-dependent sources (`NeighborBitmask`, `NeighborCondition`, `VertexCorners`) are not defined for Multihex and MUST be rejected at load time. A future revision may define them if a concrete use case emerges.

### 3.5 `FreePos`

Object lives at an arbitrary world-space pixel point. Position = `(x, y)` in world coordinates.

Used for: in-flight projectiles, world-space particles, floating damage numbers.

Neighbour-dependent sprite sources are undefined for `FreePos` and MUST be rejected at load time.

### 3.6 `Viewport`

Object lives in screen space. Its position is computed per layer from `anchor_point` (top-left, center, bottom-center, stretch, etc.) and is independent of the world camera except through `parallax`.

Used for: multi-layered skyboxes and far backgrounds, weather overlays, vignettes, letterboxing.

Viewport objects are never culled against the world grid. They are drawn as long as their object instance is alive.

## 4. Assets, Tints, Animations, Effects

These are reusable resources declared once, referenced by id from anywhere in the spec.

### 4.1 Asset

```ron
Asset(
    id: "terrain_atlas",
    path: "assets/terrain.png",
    kind: Atlas(
        tile_size: (72, 64),
        columns: 8,
        // Optional named-frame manifest: "name" -> (col, row).
        // Used by sources that reference frames by semantic name (skirts,
        // edge blends, triangle blends). Leave empty when every frame is
        // addressed by its numeric index instead.
        frames: {
            "grass_edge_n":  (0, 2),
            "grass_edge_ne": (1, 2),
            // ...
            "tri_gsw_0":     (0, 4),
            "tri_gsw_1":     (1, 4),
            "tri_gsw_2":     (2, 4),
        },
    ),
    // or:
    // kind: Single,
    // kind: SpriteSheet(frame_count: 8, layout: Horizontal),
    // kind: SpriteSheet(frame_count: 16, layout: Grid(columns: 4)),

    // Optional texture sampling parameters. All three default to the first
    // value listed below. They are asset-wide — every sprite drawn from this
    // asset uses them. For per-sprite overrides, split the source images into
    // separate Assets.
    filter: Linear,                   // Linear | Nearest           (default: Linear)
    mipmap: Off,                      // Off | Nearest | Linear     (default: Off)
    wrap:   Clamp,                    // Clamp | Repeat | Mirror    (default: Clamp)
)
```

`SpriteRef(asset_id, frame_name_or_index)` resolves to an asset and a frame within it.

For `Atlas` assets, frame lookup tries two paths in order:

1. **Named frame** — the string is checked against `Atlas.frames`. If present, its declared `(col, row)` is used to compute the pixel region.
2. **Numeric index** — the string is parsed as a non-negative integer. If successful, the grid layout computes `col = idx % columns`, `row = idx / columns`, and the region is the corresponding tile.

If neither resolves, compilation fails with an explicit error pointing at the missing frame — there are no silent placeholder regions. Autotile atlases (SPEC §5.4 `ByAtlas`) typically use numeric indices `"0".."63"` and leave `frames` empty; atlases that mix terrain bases, skirts, and triangle blends populate `frames` with semantic names.

**Sampler parameters** (`filter`, `mipmap`, `wrap`) share semantics with `tilemap_renderer::types::SamplerFilter` / `MipmapMode` / `WrapMode`:

- `filter = Nearest` is standard for pixel art; `Linear` for everything else.
- `mipmap` matters only on GPU backends and only for assets drawn at widely varying scales (parallax mountains, zoomed-out overworld). Enabling costs a small upload at load time.
- `wrap = Repeat` lets a single draw call cover an arbitrarily large area with a tileable texture — tileable sky backgrounds, long edge-anchored rivers, multi-hex seamless floors. `Mirror` is the same with per-period reflection, which hides seams in some textures cheaply.
- Non-GPU backends (SVG) ignore `mipmap` entirely and approximate `wrap` with multiple draw calls where feasible; unsupported modes fall back to `Clamp`-equivalent behaviour with a warning (see §12.2). Backend support for `wrap` lands with the rendering phase — until then, all backends treat every asset as `Clamp` regardless of the spec value.

### 4.2 Tint

```ron
Tint(
    color: "#rrggbb" | "#rrggbbaa",
    strength: 0.0..=1.0,    // 0 = identity, 1 = full replacement
    mode: Multiply | Screen | Overlay,  // default: Multiply
)
```

Referenced by `TintRef("name")`. Special symbolic tints used inside `Masked` behaviour:

- `TeamColor` — resolved at render time from the object instance's `owner` field against `Scene.players[].color`.
- `FogDependent` — resolved at render time from fog-of-war visibility state.

### 4.3 Animation

```ron
Animation(
    id: "water_flow",

    // Regular timing — all frames have the same duration derived from fps.
    frames: ["water_0", "water_1", "water_2", "water_3"],
    fps: 6.0,
    // Or pull from a sprite sheet:
    // from_sheet: ("water_sheet", 0, 8),   // (asset_id, start_frame, count)

    // Irregular timing — per-frame duration. `fps` is ignored if present.
    // frames_timed: [
    //     (sprite: "attack_0", duration_ms: 80),
    //     (sprite: "attack_1", duration_ms: 80),
    //     (sprite: "attack_2", duration_ms: 240),  // held
    //     (sprite: "attack_3", duration_ms: 100),
    // ],

    mode: Loop | PingPong | OneShot,
    phase_offset: None | HashCoord | Fixed(0.3),
)
```

An animation uses **either** `frames` + `fps`, **or** `from_sheet` + `fps`, **or** `frames_timed`. Mixing is a load-time error.

Referenced by `AnimationRef("water_flow")` or by bare string where context is unambiguous.

### 4.4 Effect

Shader-driven procedural modifications. No per-frame sprite data.

```ron
Effect(
    id: "wind_sway",
    kind: VertexDisplace(axis: X, amplitude: 2.0, frequency: 0.8),
    // Other kinds:
    // AlphaPulse(min: 0.5, max: 1.0, frequency: 2.0),
    // ColorShift(target: "#ffaa00", amplitude: 0.2, frequency: 0.5),
    phase_offset: HashCoord,
)
```

Referenced by `EffectRef("wind_sway")`.

## 5. Sprite Sources

A sprite source is a rule that, given a layer's context, produces a concrete sprite-or-frame for the current render call. Sources are polymorphic; each variant has its own inputs.

Sources split into two categories:

- **Leaf sources** — produce a sprite without contextual lookup beyond the object's own position. These are `Static`, `Variant`, `Animation`, and `External`. They compose freely (a `Variant` wraps inner leaves; an `External` returns a sprite from game code).
- **Composite sources** — emit sprites based on grid context. These are `NeighborBitmask`, `NeighborCondition`, `VertexCorners`, `EdgeConnectedBitmask`, and `ViewportTiled`. They cannot nest inside other composite sources (no neighbour-of-neighbour lookups), but their internal value slots accept leaf sources.

Concretely: where a composite source stores a "sprite per mask" or "sprite per variant", that slot accepts any leaf source. This is how an autotile mapping can point to an `Animation` (animated wall segment), or a `Variant` with `HashCoord` selection (three random-looking variations of the same wall shape), or `Variant` of `Animation` (three random-looking *animated* variations).

### 5.1 `Static`

```ron
sprite_source: Static(SpriteRef("atlas", "grass_01"))
```

Fixed sprite. No selection logic.

### 5.2 `Variant`

```ron
sprite_source: Variant(
    variants: [
        (sprite: Static(SpriteRef("atlas", "grass_01")),      weight: 5),
        (sprite: Static(SpriteRef("atlas", "grass_flowers")), weight: 2),
        (sprite: Animation(AnimationRef("rare_sparkle")),     weight: 1),
    ],
    selection: HashCoord | Random | Fixed(0),
)
```

Variant entries each wrap a **sub-source**, usually `Static` or `Animation`. The outer source picks one entry per object instance; the inner source runs every frame.

`HashCoord` is the default and requires the anchor to have a grid coordinate. `Random` resolves at scene load using the scene seed. `Fixed(i)` forces a specific variant.

### 5.3 `Animation`

```ron
sprite_source: Animation(AnimationRef("knight_idle"))
```

References a declared `Animation` resource. Current frame is selected per §7.

### 5.4 `NeighborBitmask`

Autotile lookup based on a connectivity bitmask.

```ron
sprite_source: NeighborBitmask(
    connects_with: ["stone_wall", "stone_gate", "tower"],
    source: ByMapping(
        mapping: {
            // Each value is a leaf source — Static, Animation, or Variant.
            0b000000: Static(SpriteRef("autotiles_atlas", "wall_single")),
            0b000001: Variant(
                variants: [
                    (sprite: Static(SpriteRef("autotiles_atlas", "wall_cap_n_a")), weight: 3),
                    (sprite: Static(SpriteRef("autotiles_atlas", "wall_cap_n_b")), weight: 1),
                ],
                selection: HashCoord,
            ),
            0b101010: Animation(AnimationRef("wall_magic_hum")),
            // ...
        },
        fallback: Static(SpriteRef("autotiles_atlas", "wall_default")),
    ),

    // Alternative: atlas laid out by bitmask index.
    // source: ByAtlas(asset: "wall_atlas", layout: Bitmask6),
)
```

Applicable to `Hex` anchor. For each of the cell's neighbours (in the order defined by tiling strategy), set bit `i` to 1 if the neighbour cell has an object whose id is in `connects_with`. Look up the resulting mask:

- **`ByMapping`** — explicit map from bitmask to a leaf source. Missing entries fall back to `fallback`.
- **`ByAtlas`** — the atlas is authored as a grid where the sprite at index `i` corresponds to bitmask `i`. Convenient when all 64 combinations are painted in a single atlas.

Not applicable to `Edge`, `Vertex`, `Multihex`, `FreePos`, `Viewport`.

### 5.5 `NeighborCondition`

Per-side conditional sprite emission. Covers two idioms:

- **Skirts** (3D side faces) — sprites that depict the downward-facing edges of a raised tile.
- **Edge blends** (Wesnoth-style overlaps) — sprites that show a higher-priority terrain spilling over into a neighbour of lower priority.

```ron
sprite_source: NeighborCondition(
    condition: AnyOf([
        NeighborIs(["water", "void"]),
        NoNeighbor,
    ]),
    sides: [S, SW, SE],
    sprite_pattern: "grass_side_{dir}",
    asset: "terrain_atlas",
)
```

For each side in `sides`, evaluate `condition` against that side's neighbour. If it matches, emit one sprite with `{dir}` substituted. A single layer with this source may emit up to `len(sides)` sprites per cell in one render pass.

Emitted sprites are positioned at the current cell's pixel center. Their image content may extend beyond the hex's visual bounds into the neighbour direction — this is a sprite-authoring concern, not a format concern. The renderer does not clip sprites to cell boundaries.

Condition grammar:

```
Condition ::= NeighborIs([ObjectId, ...])
            | NoNeighbor
            | NeighborPriorityLower
            | AnyOf([Condition, ...])
            | AllOf([Condition, ...])
            | Not(Condition)
```

`NeighborPriorityLower` compares against an optional integer `priority` field declared on objects (see §11). True when the current cell's object has strictly higher priority than the neighbour at the side being examined.

Applicable to `Hex`. Not applicable to other anchors.

#### 5.5.1 Idiom: Wesnoth-style edge blending

Each terrain object carries an integer `priority`. "Higher priority" terrains visually spill into "lower priority" neighbours. Grass (priority 10) next to water (priority 5) — grass draws a thin overlap on the water side of the shared edge.

```ron
Object(
    id: "grass",
    anchor: Hex,
    global_layer: "terrain",
    priority: 10,
    states: {
        "default": [
            // Base fill of the grass hex.
            Layer(
                id: "base",
                sprite_source: Variant(
                    variants: [
                        (sprite: Static(SpriteRef("terrain_atlas", "grass_01")),      weight: 5),
                        (sprite: Static(SpriteRef("terrain_atlas", "grass_flowers")), weight: 2),
                    ],
                    selection: HashCoord,
                ),
                z_in_object: 0,
            ),
            // Edge overlap: grass reaching into any lower-priority neighbour.
            // Drawn in a later pipeline pass so it layers over ALL base terrain,
            // not just this tile's base.
            Layer(
                id: "edges",
                sprite_source: NeighborCondition(
                    condition: NeighborPriorityLower,
                    sides: [N, NE, SE, S, SW, NW],
                    sprite_pattern: "grass_edge_{dir}",
                    asset: "transitions_atlas",
                ),
                pipeline_layer: "terrain_edges",
                z_in_object: 0,
            ),
        ],
    },
)
```

The recommended pipeline order for this idiom is:

```
background → terrain → terrain_edges → terrain_fx → units → ...
```

All hexes finish drawing their `base` layer (in bucket `terrain`) before any hex draws its `edges` layer (in bucket `terrain_edges`). This guarantees that grass's edge sprite at `(2, 1)` lands on top of water's already-drawn base at `(2, 2)`.

This idiom composes freely with skirts (bottom-facing 3D sides) and with dual-mesh triangles (`VertexCorners`) — you can use all three simultaneously; they are independent layers, typically each in its own pipeline bucket.

### 5.6 `VertexCorners`

Dual-mesh triangle (or square-vertex quad) lookup.

```ron
sprite_source: VertexCorners(
    patterns: [
        (corners: ("grass", "grass", "water"),    sprite_pattern: "tri_gg_w_{rot}",   priority: 10),
        (corners: ("grass", "sand",  "water"),    sprite_pattern: "tri_g_s_w_{rot}",  priority: 15),
        (corners: ("*",     "*",     "void"),     sprite_pattern: "tri_edge_{rot}",   priority: 0),
    ],
    asset: "transitions_atlas",
)
```

Applicable to `Vertex` anchor. At render time:

1. Read the object ids at the vertex's corners (from the scene).
2. Build the canonical sorted tuple and `rotation`.
3. Match against `patterns` using specificity (fewer wildcards wins) and then `priority` (§9).
4. Emit the chosen sprite with `{rot}` substituted. If no pattern matches, the vertex emits nothing.

`"*"` matches any single corner value. Wildcards participate in sorting as if they were lexicographically greater than any concrete id (so they trail in the canonical tuple).

### 5.7 `ViewportTiled`

Background / foreground textures in screen space.

```ron
sprite_source: ViewportTiled(
    content: Static(SpriteRef("bg", "sky_dusk")),   // or Animation(...)
    tiling: Stretch | Fit | Center | Repeat2D | RepeatX | RepeatY,
    anchor_point: TopLeft | TopCenter | Center | BottomCenter | BottomRight | ...,
)
```

Applicable to `Viewport` anchor only. Inner `content` is another sprite source (typically `Static` or `Animation`).

**Implementation status (Slice 4):** `Center`, `Stretch`, `Fit` are supported end-to-end (WebGL adapter). `Repeat2D` / `RepeatX` / `RepeatY` are declared but rejected at compile time with `CompileError::UnsupportedSource` — they need a `Mesh` command with texture-wrap UVs and land in a later slice. SVG / terminal adapters skip `ScreenSpaceSprite` commands silently for now.

### 5.8 `External`

Sprite chosen by game code, not by the format.

```ron
sprite_source: External(slot: "body")
```

At render time, the renderer looks up `(object_instance, slot)` in an external table populated by game code via `set_sprite(instance, slot, SpriteRef)`. If the slot is unset, the layer is skipped this frame with a warning (see §12.2).

Applicable to all anchors.

### 5.9 `EdgeConnectedBitmask`

Edge-anchored autotile: selects a sprite based on which neighbouring edges at the current edge's two endpoints carry connected objects. The edge analogue of `NeighborBitmask` (§5.4).

```ron
sprite_source: EdgeConnectedBitmask(
    connects_with: ["river_segment", "river_mouth"],
    source: ByMapping(
        mapping: {
            // Each value is a leaf source: Static, Animation, or Variant.
            // Rivers are typically animated; many masks also want HashCoord
            // variants so neighbouring straight segments look different.
            0b0000: Animation(AnimationRef("river_isolated_flow")),
            0b0101: Variant(
                variants: [
                    (sprite: Animation(AnimationRef("river_straight_a")), weight: 3),
                    (sprite: Animation(AnimationRef("river_straight_b")), weight: 2),
                    (sprite: Animation(AnimationRef("river_straight_c")), weight: 1),
                ],
                selection: HashCoord,
            ),
            0b0011: Animation(AnimationRef("river_Y_at_start")),
            0b1100: Animation(AnimationRef("river_Y_at_end")),
            0b1111: Animation(AnimationRef("river_tjunction_both_ends")),
            // ... up to 16 entries ...
        },
        fallback: Animation(AnimationRef("river_default_flow")),
    ),
    layout: EdgeHex,
)
```

Applicable to `Edge` anchor only. Not applicable to other anchors.

**Topology.** In a hex grid, exactly 3 edges meet at every vertex. An edge has 2 endpoints, so each edge has up to 4 potentially-connected neighbour edges (2 at each end) — hence a 4-bit mask.

**Bit layout `EdgeHex`.** The canonical edge `(hex, direction)` has two endpoint vertices. Using the canonical hex's direction order (§2.3), the edge sits between the direction-previous and direction-next edges (e.g., edge `N` is between `NW` and `NE`). Define:

- `start_vertex` — the endpoint shared with the clockwise-previous edge of the canonical hex (`NW` endpoint for edge `N`).
- `end_vertex` — the endpoint shared with the clockwise-next edge (`NE` endpoint for edge `N`).

At each endpoint vertex, two other edges meet. Call them the *counter-clockwise* (ccw) and *clockwise* (cw) neighbour relative to the current edge's direction from start to end:

| Bit  | Meaning                                                     |
|------|-------------------------------------------------------------|
| 0x1  | ccw neighbour at `start_vertex` carries a connected object. |
| 0x2  | cw  neighbour at `start_vertex` carries a connected object. |
| 0x4  | ccw neighbour at `end_vertex`   carries a connected object. |
| 0x8  | cw  neighbour at `end_vertex`   carries a connected object. |

**Sprite orientation.** The mask is computed in the canonical edge's frame. Implementations render the chosen sprite rotated to match the canonical edge's angle (3 unique orientations for hex flat-top/pointy-top, one per direction pair). Authors design sprites in one reference orientation; the renderer rotates. If an author wants to override rotation per mask, use `sprite_pattern` with a `{rot}` placeholder and declare rotation-specific sprites.

**Junction semantics.** At a vertex where three connected edges meet (Y-junction for rivers), each of the three edges sees both of its vertex-neighbours as connected and sets both bits at that endpoint (`0b11` on one end). Authors provide Y/T-shaped sprites for masks `0b0011`, `0b1100`, `0b1111`, etc.

**Symmetry note.** Many river masks are visually identical under left/right mirror (e.g., `0b0001` vs `0b0010`). Implementations MAY permit authors to declare one sprite for both via a convention like `symmetric: true` on the source. Not normative in 0.2.0 — duplicate entries in `mapping` work fine.

## 6. Layer Behaviour

All behaviour fields are optional. Defaults reduce to "draw the sampled sprite as-is, normal blending".

```ron
behaviour: (
    tint:    None | Flat(TintRef("dusk")) | Masked(mask_source, tint),
    blend:   Normal | Multiply | Screen | Add | Overlay,
    alpha:   1.0,
    effects: [EffectRef("wind_sway"), ...],

    // Viewport anchor only:
    parallax:        1.0,
    scroll_velocity: (0.0, 0.0),   // world pixels per second
)
```

### 6.1 Tint

- `None` — sample the sprite unmodified.
- `Flat(TintRef)` — multiply the whole sprite by the named tint.
- `Masked(mask_source, tint)` — sample a second sprite from `mask_source` and apply `tint` only where the mask's alpha is nonzero. The mask's sprite source may be any of §5 (typically `Static` or `Animation`). A `Masked` layer effectively samples two textures per draw.

`tint` inside `Masked` may be:

```
TintRef("sepia") | TeamColor | FogDependent
```

Masked `Animation` masks MUST declare a frame count compatible with the body layer for intra-object sync (§7.3).

### 6.2 Blend

Standard compositing modes over the accumulated layer stack:

| Mode | Formula |
|------|---------|
| `Normal`   | `dst = src * src.a + dst * (1 - src.a)` |
| `Multiply` | `dst = src * dst` |
| `Screen`   | `dst = 1 - (1 - src) * (1 - dst)` |
| `Add`      | `dst = src + dst` (clamped) |
| `Overlay`  | Combined multiply/screen per channel |

### 6.3 Effects

References to declared `Effect` resources (§4.4). Applied after sampling and tinting.

### 6.4 Parallax and Scroll (Viewport only)

- `parallax: f32` — factor applied to the camera's world offset when computing the layer's screen position. `0.0` pins to screen, `1.0` moves with the world, values between produce depth, `>1.0` produces foreground parallax.
- `scroll_velocity: (f32, f32)` — autonomous pixel-per-second drift independent of the camera. Adds to the texture offset every frame.

Using these fields on non-`Viewport` anchors MUST be a load-time error.

### 6.5 Alpha

Static scalar 0..1 applied before blending. `AlphaPulse` effect modulates on top.

## 7. Animations and Synchronization

### 7.1 Local time

Every animated source has a local time `t_local`:

```
t_local = t_global + phase_offset
```

`phase_offset` is:

- `None` — zero.
- `Fixed(seconds)` — constant.
- `HashCoord` — `(hash_coord(q, r, salt) / u32::MAX) * animation_period`, where `salt = hash_str(animation.id)`. Requires a grid-anchored anchor.

### 7.2 Frame selection

Given `t_local` and the animation definition:

- Regular timing: `frame_index = floor(t_local * fps) mod len(frames)` for `Loop`; equivalent logic for `PingPong` and `OneShot`.
- Irregular timing: accumulate `duration_ms` until reaching `t_local`; walk the frame list.
- `OneShot` completes and stops on its last frame; implementations SHOULD expose a completion callback to game code.

### 7.3 Intra-object sync

**By default, all animated layers within the same object instance share `t_global`** (the instance's birth time or a shared clock chosen by the implementation). Two layers with the same frame count and no explicit `phase_offset` play in lock-step. This is how a unit body and its team-color mask stay aligned.

Explicit `phase_offset` on a layer decouples it from the object's shared clock (e.g., a blinking detail with `Fixed(0.5)` offset).

### 7.4 Inter-object sync

There is no implicit inter-object synchronization. All objects at `phase_offset: None` read `t_global` from the renderer's master clock, so they *happen* to be synchronized, but this is not a guarantee — instance-local clocks are permitted.

Use `phase_offset: HashCoord` on grid-anchored animations (water, trees, torches) to spread neighbours across the phase space so that pulses do not march in unison.

## 8. Pipeline

```ron
pipeline: RenderPipeline(
    tiling: HexFlatTop,
    grid_stride: (72, 64),

    viewport_size: (1280, 720),   // optional; derived from window if absent
    clear_color: Some((0.05, 0.07, 0.12, 1.0)),  // optional linear RGBA; None = transparent

    // Bottom-to-top list of z-buckets. Objects reference these by name via
    // `global_layer: "background"`.
    layers: [
        Layer(id: "background",     sort: None,  tint_mask: None),
        Layer(id: "terrain",        sort: None,  tint_mask: None),
        Layer(id: "terrain_edges",  sort: None,  tint_mask: None),  // Wesnoth-style overlaps
        Layer(id: "terrain_fx",     sort: None,  tint_mask: None),  // dual-mesh blends, skirts, autotile overlays
        Layer(id: "units",          sort: YAsc,  tint_mask: None),
        Layer(id: "world_fx",       sort: None,  tint_mask: None),
        Layer(id: "weather_fg",     sort: None,  tint_mask: None),
        Layer(id: "ui",             sort: None,  tint_mask: None),
    ],

    global_tint: Some(TintRef("dusk")),
)
```

### 8.1 Layer ids

Layer ids are user-chosen. The above is an example, not a requirement. The renderer does not distinguish `"units"` from `"terrain"` semantically — it only obeys the declared order.

### 8.2 Sort modes

- `None` — draw in the order object instances were spawned (deterministic for static scenes).
- `YAsc` — sort by screen Y ascending (objects further down the screen draw later and appear in front).
- `YDesc` — reverse.

For `Multihex`, the Y used is `sort_y_source` (anchor Y by default, bottom-of-shape Y if configured on the object).

### 8.3 Per-layer bucket override

A `Layer` may carry an optional `pipeline_layer: "<bucket_id>"` that overrides its parent object's `global_layer` for that one layer. This lets a single object contribute draw calls to multiple pipeline passes. The canonical use is Wesnoth-style edge transitions (§5.5.1): the terrain object's base layer draws in `terrain`, its edge-blend layer draws in `terrain_edges`, so edge overlaps land on top of *all* base terrain across the map.

Within one pipeline bucket, all draw calls from all contributing objects are gathered, the bucket's `sort` mode is applied, and they are submitted in order. `z_in_object` provides a deterministic tiebreaker when two draw calls share a sort key.

### 8.4 Global tint

Applied multiplicatively to every draw call after all per-object tints and effects. Typical use: time-of-day.

## 9. Rule Specificity and Priority

Applies to `NeighborCondition` and `VertexCorners` sources, which may have multiple matching rules for the same input.

Resolution order:

1. **Specificity**: rule with fewer wildcards wins.
2. **Priority**: higher integer `priority` wins.
3. **Declaration order**: earlier entry wins.

Implementations SHOULD emit a warning when two rules match with equal specificity and equal priority.

## 10. File Structure

> **Data structure first, file format second.** Both `RenderSpec` and `Scene`
> are plain Rust structs with `serde` derives. The RON payload described below
> is one serialisation; JSON and any other `serde`-compatible format work
> identically. Games that already have their own scene / map representation
> (JSON, binary, ECS queries) are expected to build `Scene` directly in memory
> from their own data — never go through a RON file at runtime. The scene
> format carries **rendering information only**; game mechanics (HP, AI, items)
> live in the game's own types.

### 10.1 `render_spec.ron`

```ron
RenderSpec(
    version: "0.2.0",
    assets:     [ Asset, ... ],
    tints:      [ Tint, ... ],        // List, not map — consistent with other collections
    animations: [ Animation, ... ],
    effects:    [ Effect, ... ],
    objects:    [ Object, ... ],      // all renderable object classes
    pipeline:   RenderPipeline,
)
```

### 10.2 `Object` structure

```ron
Object(
    id: "knight",
    anchor: Hex,
    global_layer: "units",

    // Optional integer used by NeighborPriorityLower comparisons.
    priority: 0,

    // Optional Multihex-specific Y-sort source.
    // sort_y_source: Anchor | BottomOfShape,

    default_state: "idle",
    states: {
        "idle": [
            Layer(
                id: "body",
                sprite_source: Animation(AnimationRef("knight_idle")),
                behaviour: (),
                z_in_object: 0,
            ),
            Layer(
                id: "team_mask",
                sprite_source: Animation(AnimationRef("knight_idle_mask")),
                behaviour: (
                    tint: Masked(Animation(AnimationRef("knight_idle_mask")), TeamColor),
                    blend: Normal,
                ),
                z_in_object: 1,
            ),
        ],
        "walk":   [ ... ],
        "attack": [ ... ],
        "die":    [ ... ],
    },
)
```

### 10.3 `scene.ron`

```ron
Scene(
    // `meta` is optional plumbing — both fields default to `None` for
    // runtime-constructed scenes; set them when loading from disk.
    meta: (
        name: Some("Demo Battle"),
        render_spec: Some("render_spec.ron"),
    ),
    bounds: (min: (-5, -5), max: (10, 10)),

    // Tiles are no longer typed — a tile is a list of object ids present on it.
    // Equivalent ASCII form below.
    tiles: [
        Tile(pos: (0, 0), objects: ["grass"]),
        Tile(pos: (1, 0), objects: ["grass", "village"]),
        Tile(pos: (2, 0), objects: ["grass", "stone_wall"]),
    ],

    // ASCII grid form:
    palette: {
        '.': ["grass"],
        '~': ["water"],
        '#': ["grass", "stone_wall"],
        'v': ["grass", "village"],
        'f': ["grass", "forest"],
    },
    map: [
        "..v#..",
        ".f.#..",
        "~....f",
    ],

    // Edge-anchored instances: one entry per canonical edge.
    edges: [
        EdgeInstance(at: (hex: (2, 1), dir: NE), object: "fence"),
    ],

    // Vertex-anchored objects are normally implicit (dual-mesh triangles
    // emit automatically from the terrain configuration); explicit entries
    // override automatic ones.

    // Multihex instances.
    multihex_instances: [
        MultihexInstance(anchor: (4, 2), object: "castle_2x2"),
    ],

    // FreePos instances.
    free_instances: [
        FreeInstance(pos: (120.5, 240.0), object: "floating_damage_10"),
    ],

    // Viewport objects — one active per spec or stacked by global_layer.
    viewport_instances: [
        ViewportInstance(object: "sky_background", animation: "dusk"),
        ViewportInstance(object: "weather_rain"),
    ],

    // Animate-ables that are "the player's pieces" — same schema as tile objects
    // but tracked separately because they move at runtime.
    entities: [
        Entity(at: (1, 1), object: "knight", owner: 0, animation: "idle"),
        Entity(at: (5, 3), object: "archer", owner: 1, animation: "idle"),
    ],

    players: [
        Player(id: 0, color: "#cc2233", name: "Red"),
        Player(id: 1, color: "#2266cc", name: "Blue"),
    ],

    // Optional runtime knobs read by the renderer on load.
    // Games can change them at runtime via API calls.
    initial_global_tint: "dusk",
)
```

`tiles` and `(palette, map)` are mutually exclusive for hex cells; exactly one MUST be present.

### 10.4 Palette entries

```
palette_entry ::= [object_id, ...]    // objects stacked on this cell, bottom-to-top
```

The ASCII grid is interpreted in offset coordinates and converted to axial internally by the tiling strategy.

## 11. Object Ids and Cross-References

- Every object declared in `render_spec.objects[]` MUST have a unique `id`.
- References from scenes (`tiles[].objects`, `entities[].object`, etc.) MUST resolve to a declared object id.
- `NeighborBitmask.connects_with`, `NeighborIs(...)`, and `NeighborCondition` terrain checks compare against object ids present on neighbour cells.
- `VertexCorners.patterns[].corners` matches against the **lowest-indexed object id on the cell that has a `priority` field set**, conventionally the terrain object. Implementations SHOULD document this resolution rule in validation errors when no corner is matchable.

## 12. Rendering Algorithm

```
for each layer in pipeline.layers:
    draw_calls = []

    for each object instance assigned to this layer:
        if instance is culled: continue
        stack = instance.object.states[instance.current_state]
        for each Layer in stack ordered by z_in_object:
            sprite = sample_source(Layer.sprite_source, instance.context, t_global)
            for each emitted sprite from NeighborCondition / VertexCorners sources:
                draw_calls.push(apply_behaviour(Layer.behaviour, sprite, instance))

    if layer.sort == YAsc: draw_calls.sort_by(|d| d.screen_y)
    elif layer.sort == YDesc: draw_calls.sort_by(|d| -d.screen_y)

    for each draw_call:
        final = compose_tints(draw_call, global_tint)
        submit_to_gpu(final)
```

### 12.1 Tint composition order

1. Sampled sprite pixels.
2. Layer behaviour (Flat or Masked tint).
3. Layer-level `effects` producing color modulations.
4. Layer's `pipeline.Layer.tint_mask`.
5. `pipeline.global_tint`.

Each step uses its own blend mode. Default is `Multiply`.

### 12.2 Missing-sprite handling

If a sprite reference at render time cannot be resolved (`External` slot unset, asset missing, mapping not found without fallback):

- Log a warning with layer id and context.
- Render a placeholder (magenta checkerboard) at the intended destination.
- Continue with remaining draw calls.

## 13. Hash Function (normative)

Used by `HashCoord` selectors for determinism across runs and platforms.

```rust
fn hash_coord(q: i32, r: i32, salt: u32) -> u32 {
    let mut h = (q as u32).wrapping_mul(73_856_093)
        ^ (r as u32).wrapping_mul(19_349_663)
        ^ salt.wrapping_mul(83_492_791);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    h
}

fn hash_str(s: &str) -> u32 {
    // FNV-1a 32-bit over bytes of s
    let mut h: u32 = 0x811c_9dc5;
    for b in s.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}
```

## 14. Renderer Integration API (contract, not normative in format terms)

The runtime API a game uses to drive the renderer. Only the semantic contracts are fixed here; concrete signatures belong to implementation docs.

- `spawn(object_id, placement) -> ObjectInstance` — placement is anchor-specific payload (cell coord for Hex, pair for Edge, pixel point for FreePos, nothing for Viewport).
- `despawn(instance)` — removes from the scene.
- `set_state(instance, name)` — switches the active state of an instance. `name` must exist in the object's `states` map.
- `set_sprite(instance, slot, sprite_ref)` — populates an `External` source.
- `set_global_tint(tint_ref | None)` — changes pipeline-level tint at runtime (day/night cycle).
- `set_camera(world_center, zoom)` — used for culling and parallax computation.

The renderer MUST treat instance state as opaque and game-owned; it does not invent states or spawn objects.

## 15. Special Ids and Edge Cases

### 15.1 `"void"`

Reserved object id meaning "off-map or unset". Used by:

- `NeighborCondition` rules — `NeighborIs(["void"])` matches map edges.
- `VertexCorners` corner tuples at map boundaries.
- ASCII palette — `' '` (space) maps to `"void"` by default if not overridden.

`"void"` MUST NOT be declared as a user object.

### 15.2 Missing states

If `set_state(instance, name)` is called with a `name` not present in the object's `states` map, the renderer MUST:

- Log a warning.
- Fall back to `default_state`.
- Continue rendering.

### 15.3 Cycles

- Tints reference no other tints.
- Animations do not contain other animations.
- Masked behaviours reference sprite sources; those sources MUST NOT themselves be Masked (no nested masks in 0.2.0).

Violations are load-time errors.

## 16. Validation Requirements

At load time, the implementation MUST verify and report all violations (not stop at the first):

- Every `SpriteRef(asset_id, _)` resolves to a declared asset.
- Every `TintRef`, `AnimationRef`, `EffectRef` resolves.
- Every `objects[].id`, `assets[].id`, `animations[].id`, `effects[].id`, `tints[].id` is unique within its collection.
- Every `NeighborBitmask.connects_with` entry is a declared object id or `"void"`.
- Pipeline layer ids are unique and non-empty; every `Object.global_layer` references a declared layer.
- For each object: `default_state` exists in `states`; every referenced animation frame count is consistent with masks sharing its slot.
- Anchor-source compatibility (see §3 and §5).
- `tiling` is one of the supported values for the current implementation version.

## 17. Versioning

`RenderSpec.version` uses semver. Implementations MUST reject specs with a major version higher than supported. Minor version additions (new anchor types, new sources) SHOULD be backward compatible; breaking changes require a major bump.

## 18. Open Points (non-blocking)

Items deliberately left under-specified in 0.2.0; MAY be tightened in 0.2.x.

1. Canonicalization tie-breakers for `Vertex` when two cells share minimum coordinates (cannot occur with axial hex coordinates but stated for completeness).
2. Exact culling margins for `Multihex` near viewport edges — implementations may pick their own.
3. `ViewportInstance` stacking behaviour when two instances share a `global_layer` — current rule: draw in declared order, no sort.
4. `External` slot types beyond `SpriteRef` (e.g., passing a tint at runtime). Deferred.
5. Square-grid tiling strategies (`Square4`, `Square8`). Reserved but not implemented.
6. Shared animation clocks across objects (e.g., torches in a village pulsing in unison with a seeded offset). Current workaround: `phase_offset: Fixed` with a coordinated value. A dedicated mechanism may be added later.
7. `HashCoord` extension for non-`Hex` anchors. The §13 hash function takes `(q, r, salt)`. `Edge` anchors need the direction index folded into the salt (e.g., `salt ^ dir_index.wrapping_mul(0x9e37_79b9)`) to give each edge a distinct variant draw; `Vertex` anchors need the vertex identifier folded in similarly. The exact formula MUST be nailed down in `spec_types.rs` so variant selection is deterministic and reproducible across runs. Hex anchors use the base formula unchanged.
8. Leaf-source generalization for `NeighborCondition.sprite_pattern` and `VertexCorners.sprite_pattern`. Today these sources carry a string template with `{dir}` / `{rot}` substitution, which resolves only to a single sprite id. To enable animated skirts (shoreline waves), animated tri-blends, and coordinate-hashed variants of either, the pattern slot should accept a leaf source whose `SpriteRef` / `AnimationRef` id is formed by the same template substitution. Design sketch: `sprite_pattern: LeafPattern(source_template)` where the template is resolved per emission. Not a blocker — static patterns cover the common case.
