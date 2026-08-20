# line_tools — test suite

## Organization

`tests.rs` is the integration-test entry point; it declares a single
`mod webgl;` and pulls in `minwebgl`/`test_tools` fixtures shared across the
suite. All current test content lives under [`webgl/`](webgl/readme.md),
which carries its own Responsibility Table since it is presently the only
domain covered — see that file for the per-test-file breakdown (distance,
point, and dash behavior of the basic line).

## Directory structure

```
tests/
  tests.rs        — entry point, declares mod webgl
  webgl/
    readme.md      — per-file Responsibility Table for this domain
    mod.rs         — module declaration
    distance.rs    — line distance functionality
    points.rs      — point add/remove operations
    dash.rs        — dash pattern/offset/toggle configuration
```

## Adding new tests

- A new `webgl`-domain test file: add it under `webgl/` and register it in
  `webgl/mod.rs`, then add a row to `webgl/readme.md`.
- A new non-`webgl` domain (e.g. a future `d3` variant per the crate's
  straddling-stacks classification): add a sibling module declared in
  `tests.rs`, with its own `readme.md`.
