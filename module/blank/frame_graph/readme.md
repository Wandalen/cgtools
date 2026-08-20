# frame_graph

Reserved crate for the L2 frame-orchestration layer of the cgtools rendering
architecture — pass scheduling, render-target lifecycles, and resolve/composite
chains, extracted from the stack engines once a second engine needs to share
them. No implementation yet; today this logic lives embedded inside the L3
engines.

Workspace-level documentation of the slot:

- `docs/layer/003_l2_frame_orchestration.md` — the layer's role and extraction trigger
- `docs/adr/001_multi_stack_rendering_architecture.md` — the architecture this crate serves
