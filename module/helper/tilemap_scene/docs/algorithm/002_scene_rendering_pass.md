# Algorithm: Scene Rendering Pass

### Scope

- **Purpose**: Walk a scene's pipeline buckets once per frame, producing a sorted, tint-composed draw-call list.
- **Responsibility**: Document the per-bucket gather/sort/submit walk, the 5-step tint composition order, missing-sprite failure semantics, and the cache-replay optimization that skips the walk on an unchanged frame.
- **In Scope**: Per-layer sprite sampling and behaviour application, bucket-level sorting, tint composition order, missing-sprite failure semantics (unset-`External` skip vs hard `CompileError`), `Renderer`'s idle-frame cache replay.
- **Out of Scope**: How an individual sprite source resolves to a sprite (see `format/005`, `algorithm/001`); how a bucket's declared sort mode is defined (see `format/007`).

### Abstract

Once per frame, the renderer turns a `Scene`'s live instances into a submitted, correctly-ordered, correctly-tinted stream of draw calls. The walk is bucket-scoped — every pipeline bucket (see `format/007`) is processed independently, in the pipeline's own declared order, so an object contributing layers to more than one bucket (via `pipeline_layer` overrides) has its draw calls fully separated and independently sorted per bucket rather than as one combined per-object unit. Layered on top of the walk itself is a cache-replay optimization: an unchanged frame (no scene mutation, no clock advance, no camera change) skips the walk entirely and replays the previous frame's command buffer, which is why this algorithm's cost is properly understood as "per changed frame," not strictly "per frame."

### Algorithm

For each pipeline bucket, in declared order (see `format/007`):

```
for each layer in pipeline.layers:
    draw_calls = []
    for each object instance assigned to this bucket:
        if instance is culled: continue
        stack = instance.object.states[instance.current_state]
        for each Layer in stack ordered by z_in_object:
            sprite = sample_source(Layer.sprite_source, instance.context, t_global)
            for each emitted sprite from NeighborCondition / VertexCorners sources:
                draw_calls.push(apply_behaviour(Layer.behaviour, sprite, instance))
    if bucket.sort != None: draw_calls.sort_by(bucket.sort's key, see format/007)
    for each draw_call:
        final = compose_tints(draw_call, global_tint)
        submit_to_gpu(final)
```

A layer whose `sprite_source` is a composite source (`NeighborCondition`, `VertexCorners` — see `format/005`) may push more than one draw call per instance in a single pass (up to `len(sides)` for `NeighborCondition`); every other source pushes exactly one.

**Tint composition order** — each draw call's final color is composed through five stages, each with its own blend mode (default `Multiply`):

1. Sampled sprite pixels (the raw texture read).
2. Layer behaviour tint (`TintBehaviour::Flat` or `::Masked`, see `format/006`).
3. Layer-level `effects` producing color modulations (see `format/004`).
4. The bucket's own `PipelineLayer.tint_mask`, if set (see `format/007`).
5. `RenderPipeline.global_tint` (see `format/007`), applied last, uniformly across every draw call in the frame regardless of which bucket or object it came from.

**Missing-sprite handling**: there is no render-time warning or placeholder path — the crate has no logging dependency at all, and the originally-specified warn-and-substitute behavior (magenta checkerboard) is a deliberately-unimplemented pending option tracked in `roadmap.md`'s `External` sprite-source item, not current behavior. What actually happens splits by cause:

- **Unset `External` slot** (game code has not called `set_external_sprite` yet — the one condition unknowable at load time): that layer of that instance silently emits nothing this frame and the pass continues (`src/compile/frame.rs`, `compile_instance_layer`'s unset-slot early return and its free-pos counterpart). Pinned by `tests/renderer_test.rs`'s "unset External slot must not emit any Sprite" assertion.
- **Any reference that fails to resolve during the walk** — a *set* `External` slot whose `(asset, frame)` pair was never pre-allocated at `Renderer::new` (External refs are not themselves pre-allocated, see `src/compile/assets.rs`, so the pair must be reachable from some other declared source or animation), an unresolvable tint or animation, an unparseable hex color — aborts the entire pass: `render()` returns `Err(CompileError::UnresolvedRef { .. })` and no command buffer is produced for that frame.
- **Autotile mask miss cannot degrade at all**: `NeighborBitmaskSource::ByMapping`'s `fallback` is a required field (`Box<SpriteSource>`, not an `Option` — see `format/005`), so a mask absent from `mapping` always resolves the declared fallback source.

Both this walk and `format/004`'s load-time frame lookup therefore fail hard on an unresolvable reference; the only deliberate leniency anywhere is the unset-`External` skip.

**Cache replay** (an optimization layered on top of the walk above, not part of the format's own contract): `Renderer::render` computes a signature from `(scene_revision, clock, camera_signature)` before doing any of the above; if the signature exactly matches the previous call's, the previously-built command buffer is returned unchanged (an idle-frame replay, counted via `cache_hits()`) and the gather/sort/compose walk above is skipped entirely for that call. Any scene mutation (`spawn`, `set_state`, `set_tint`, `tick` advancing the clock, etc. — see `api/001`) changes `scene_revision` or `clock` and invalidates the cache for the next call.

### APIs

| File | Relationship |
|------|--------------|
| [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) | `Renderer::render(&scene, &camera)` is the entry point for this algorithm; scene mutators invalidate the cache-replay signature |

### Formats

| File | Relationship |
|------|--------------|
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | Culling ("if instance is culled: continue") is anchor-specific |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | `sample_source` step; composite sources may emit multiple draw calls per instance |
| [format/006_layer_behaviour.md](../format/006_layer_behaviour.md) | `apply_behaviour` step; stages 2–4 of the tint composition order |
| [format/007_render_pipeline.md](../format/007_render_pipeline.md) | Bucket order and sort mode consumed by this algorithm |

### Sources

| File | Relationship |
|------|--------------|
| `src/renderer.rs` | `Renderer::render`, `cache_hits`, `cleanup`, signature/cache-replay logic |
| `src/compile/frame.rs` | Per-frame emit gathering (`gather_frame_emits`) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/renderer_test.rs` | End-to-end render pass coverage |
| `tests/renderer_cache_test.rs` | Cache-replay signature invalidation coverage |
| `tests/sorted_batching_test.rs` | Bucket sort-mode application |
