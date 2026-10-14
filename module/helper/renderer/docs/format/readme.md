# Format Doc Definition

A **format** instance documents one piece of this crate's on-disk or data-file contract. In `renderer`, that's what an engravable glTF asset and its `engraving_config.json` must look like for the text-engraving runtime. This collection holds one instance per format concern; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for the asset and data-file contracts `renderer` consumes beyond plain glTF.
- **Responsibility**: Document each contract's required structure, fields and authoring rules.
- **In Scope**: Engravable mesh and UV layout, glTF export settings, `engraving_config.json`.
- **Out of Scope**: Runtime behavior operating on this data (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Engraving Asset Pipeline](001_engraving_asset_pipeline.md) | Mesh topology, dual UV channels, export settings and `engraving_config.json` for engravable models | ✅ |
