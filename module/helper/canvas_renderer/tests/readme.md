# Tests

Native tests for `canvas_renderer`'s pure-logic surface, runnable without a browser.
Everything GL-bound (every `CanvasRenderer` method takes `&GL`) waits on the
workspace's wasm test-runner infrastructure.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| renderer_test.rs | Verifies mesh-to-color resolution across non-mesh siblings |
