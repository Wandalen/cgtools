# Invariant Doc Definition

An **invariant** is a guarantee this crate enforces and callers may rely on. In `gpu_hal`, these are the crate-wide error contract plus two WebGL-specific call-order guarantees that fall out of WebGL's eager, introspection-based binding resolution — stated once, explicitly, along with how each is enforced. This collection holds one instance per invariant, each pinned to where it is enforced in code; the table below is the index into them.

### Scope

- **Purpose**: The crate's error-handling contract and its WebGL-specific ordering guarantees hold across every module and are worth stating once, explicitly.
- **Responsibility**: Document crate-wide invariants and their enforcement mechanisms.
- **In Scope**: Error handling contract, unsafe-code prohibition, scoped panic policy, WebGL render-pass recording order, WebGL bind-group entry order.
- **Out of Scope**: Per-module error variant wording (read `src/error.rs` directly).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Result-Based Error Handling with a Scoped Panic Policy](001_result_based_error_handling_scoped_panics.md) | Every fallible op returns `Result<_, Error>`; zero `unsafe`; panics confined to `pub(crate)` backend-mismatch accessors | ✅ |
| 002 | [WebGL Render-Pass Recording Order](002_webgl_render_pass_recording_order.md) | `pipeline_set` must precede `bind_group_set`/`vertex_buffer_set` | ✅ |
| 003 | [WebGL Bind-Group Entry Order](003_webgl_bind_group_entry_order.md) | A `Sampler` entry pairs with the nearest preceding `Texture` entry of its group | ✅ |
