# d3_scene

Reserved crate for the L4/L5 scene layer of the d3 rendering stack — a
declarative, deterministic scene model plus scene-as-script runners over the
`renderer` engine, following the working pattern `tilemap_scene` established
for the tile stack. No implementation yet; creation is gated on a committed
d3 scene-file requirement.

Workspace-level documentation of the slot:

- `docs/layer/005_l4_scene_model.md` and `docs/layer/006_l5_scene_script_and_runners.md` — the layers this crate will occupy
- `docs/render_stack/003_d3.md` — the stack whose invariants it must assume
- `docs/adr/001_multi_stack_rendering_architecture.md` — the architecture this crate serves
