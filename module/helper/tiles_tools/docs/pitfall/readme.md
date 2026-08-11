# Pitfall Doc Definition

A **pitfall** documents one way this crate's API can be misused or misunderstood — the trap, why it happens, and how to avoid it. In `tiles_tools`, that covers known traps in the current implementation — unimplemented flow fields, no-op ECS movement requests, non-functional save-file compression, and hexagonal distance method ambiguity — each written down with its failure mode and mitigation. This collection holds one instance per known pitfall; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for known traps in `tiles_tools`' current implementation.
- **Responsibility**: Document each trap's failure mode and mitigation.
- **In Scope**: Unimplemented flow fields, no-op ECS movement requests, non-functional save-file compression, hexagonal distance method ambiguity.
- **Out of Scope**: Working alternatives to each trap — cross-referenced from each individual pitfall file, not duplicated here.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Flow Field Algorithm Unimplemented](001_flow_field_algorithm_unimplemented.md) | Every `FlowField`/`IntegrationField` method returns a fixed stub value | ✅ |
| 003 | [Save-File Compression Is a Fake Wrapper](003_savefile_compression_is_a_fake_wrapper.md) | `compress_data` adds a 7-byte header without shrinking anything | ✅ |
| 004 | [Hexagonal Axial Distance Method Ambiguity](004_hexagonal_axial_distance_method_ambiguity.md) | Two same-named `distance` methods resolve differently by argument shape | ✅ |
