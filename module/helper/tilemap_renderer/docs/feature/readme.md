# Feature Doc Entity

### Scope

- **Purpose**: `tilemap_renderer`'s backend adapters exist to let one command stream render to SVG, WebGL2, or a terminal.
- **Responsibility**: Document each backend adapter as a navigational hub over its source, invariants, patterns, and known pitfalls.
- **In Scope**: The three feature-gated `Backend` implementations shipped (or stubbed) in this crate.
- **Out of Scope**: The shared core command/asset vocabulary, which is not adapter-specific (see `src/types.rs`, `src/commands.rs`, `src/assets.rs` directly, and [pattern/001](../pattern/001_ports_and_adapters_backend_architecture.md) for the trait boundary).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [SVG Backend Adapter](001_svg_backend_adapter.md) | Generates SVG 1.1 documents from a command stream | ⚠️ |
| 002 | [WebGL2 Backend Adapter](002_webgl2_backend_adapter.md) | Hardware-accelerated sprite/mesh/batch rendering on `wasm32` | ⚠️ |
| 003 | [Terminal Backend Adapter](003_terminal_backend_adapter.md) | ASCII/Unicode terminal rendering | ⏸️ |
