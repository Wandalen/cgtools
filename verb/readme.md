# verb

Verb modules for the cgtools workspace, resolved by the ancestor-walk `verb`
dispatcher (or invoked directly by path).

| File/Directory | Responsibility |
|----------------|----------------|
| `install/` | Install all workspace binaries (module/ bin crates): `verb/install/run [dry::1] [crate ...]`. Directory-shaped module, `inheritable = false` — resolves via the `verb install` dispatcher from the workspace root only, never from a nested directory. |
| `test` | Full/final verification gate: native suite (nextest + doctests + clippy) plus wasm32 compile check and browser-driven wasm test suites. |
| `test_only` | Ordinary scoped verification during development: native nextest, narrowed via `pkg::<crate>` or a filter expression. |
