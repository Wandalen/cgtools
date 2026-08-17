# Format :: F02. tree_aligned

| Field | Value |
|-------|-------|
| ID | F02 |
| Output context | `tree`, both with a `name` argument (single dependency tree) and without one (a forest — one tree per root, printed back-to-back) |
| Trigger | `shape::aligned` (the default — omitting `shape::` entirely also selects it) |
| Structure | One line per chunk node, child chunks nested and indented under their parent showing the `depends_on` relationship, each node's tags carried as trailing column data aligned across the whole tree |
| Rendering mechanism | `data_fmt`'s `TreeNode<ColumnData>` built by recursively walking `depends_on`, rendered via `TreeFormatter::format_aligned()` |
| Example | See below |

### Example

`tree fbm3` (dependency chain `fbm3 → value_noise → hash21`):

```text
└── fbm3             category:noise, technique:fractal
    └── value_noise  category:noise
        └── hash21   category:hash
```

`tree` with no argument (forest of every chunk nothing else depends on —
42 roots in the current registry; excerpt below):

```text
└── fullscreen_triangle  category:vertex

└── hash33  category:hash
...
```

Exact indentation/branch glyphs and column alignment are produced by
`TreeFormatter::format_aligned()` at render time — the shape above
(parent-before-child, indented, tags trailing) is the documented contract;
precise glyph choice is an implementation detail of `data_fmt`.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.tree](../command/04_tree.md) | Single tree (with `name`) or forest (without) |

---

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`name`](../param/01_name.md) | Selects a single tree's root; omitted selects the full forest |
| 2 | [`reverse`](../param/22_reverse.md) | Flips edge direction from dependencies to dependents |
| 3 | [`shape`](../param/24_shape.md) | `aligned` selects this format (the default) |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
