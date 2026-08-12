# Layer: L2 Frame Orchestration

Pass scheduling, render-target lifecycles, and resolve/composite chains —
the machinery that turns "draw these things" into an ordered frame. Today
this layer exists only *embedded inside* each L3 engine; the blank crate
`frame_graph` reserves the slot for its extraction.

### Scope

- **Purpose**: Define the frame-orchestration layer's role and record where its logic currently lives.
- **Responsibility**: Name the embedded instances, the extraction trigger, and the reservation state.
- **In Scope**: Target allocation/lifecycle, pass ordering, resolve and composite steps.
- **Out of Scope**: What the passes draw (stack-engine concern, see [004_l3_stack_engine.md](004_l3_stack_engine.md)); the GPU API the targets are allocated through (see [002_l1_gpu_hal.md](002_l1_gpu_hal.md)).

### Role

An L2 crate owns *frame shape*: which targets exist (formats, multisampling),
in what order passes run, when multisampled targets resolve, and how partial
results composite. It is stack-agnostic machinery — a d2 engine and a d3
engine schedule very different passes, but allocation, dependency ordering,
and resolve mechanics are the same shape of problem.

### Embedded Instances Today

- `renderer` (`src/webgl/renderer.rs`): the MSAA `RGBA16F` target set
  (main / emission / transparent accumulate / revealage), the opaque →
  transparent → `resolve` → post-chain ordering, and the
  `post_processing/pass.rs` pass composition.
- `tilemap_renderer` (WebGL2 adapter): per-batch VAO lifecycle and
  draw-time state management inside `src/adapters/webgl.rs`.

### Extraction Trigger

Extract into `frame_graph` only when a second engine needs to *share* pass
logic — sharing, not symmetry, is the trigger. Until then, embedded is the
correct state (YAGNI); the slot exists so the eventual extraction has a
name and a documented home.

### Layers

| File | Relationship |
|------|--------------|
| [002_l1_gpu_hal.md](002_l1_gpu_hal.md) | The layer L2 will allocate targets and pipelines through |
| [004_l3_stack_engine.md](004_l3_stack_engine.md) | The engines currently embedding this layer's logic |

### Sources

| File | Relationship |
|------|--------------|
| `module/blank/frame_graph/` | The reserved crate slot |
| `module/helper/renderer/src/webgl/renderer.rs` | The richest embedded instance: target zoo + pass ordering + resolve |
| `module/helper/renderer/src/webgl/post_processing/pass.rs` | Pass composition machinery |
