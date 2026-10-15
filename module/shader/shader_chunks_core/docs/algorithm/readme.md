# Algorithm Doc Definition

An **algorithm** here is a step-by-step procedure this crate executes, with
correctness properties worth stating explicitly. Two exist: the build-time
generation of the bundled-chunk registry, and the topological composition
that turns any chunk set into one WGSL source. This collection holds one
instance per procedure; the table below is the index into them.

### Scope

- **Purpose**: Document the two procedures the crate's guarantees flow through — how the registry comes to exist and how a set becomes a shader.
- **Responsibility**: Describe each procedure's inputs, steps, ordering/failure properties, and a worked example.
- **In Scope**: Registry generation (`build.rs`) and dependency-ordered composition (`compose*`/`entries_sort_and_join`).
- **Out of Scope**: The properties these procedures enforce (see `invariant/`); the consumer-side forms that call them (see `pattern/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Registry Generation](001_registry_generation.md) | Derive `CHUNKS` + `chunk_get` from the `shader/` collection at build time, cross-validated both ways | ✅ |
| 002 | [Dependency-Ordered Composition](002_dependency_ordered_composition.md) | Topologically sort a chunk set by `depends_on` and concatenate dependency-before-dependent | ✅ |
