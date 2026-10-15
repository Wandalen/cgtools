# Invariant Doc Definition

An **invariant** is a guarantee this crate enforces and callers may rely on.
In `shader_chunks_core`, that covers the completeness of any composed chunk
set and the field-level truth of every descriptor. This collection holds one
instance per invariant, each pinned to where it is enforced in code; the
table below is the index into them.

### Scope

- **Purpose**: Navigational hub for the correctness properties the compile-time chunk machinery must always uphold.
- **Responsibility**: Document each invariant's precise statement, enforcement mechanism, and violation consequences.
- **In Scope**: Dependency closure of composed sets; descriptor-manifest field parity.
- **Out of Scope**: The procedures that operate under these guarantees (see `algorithm/`); the consumer forms that invoke the enforcement (see `pattern/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Dependency Closure](001_dependency_closure.md) | Every `depends_on` entry of every set member resolves within the set — checkable at compile time | ✅ |
| 002 | [Descriptor-Manifest Parity](002_descriptor_manifest_parity.md) | Every descriptor field equals what the parsers read from its own manifest — bundled and local alike | ✅ |
