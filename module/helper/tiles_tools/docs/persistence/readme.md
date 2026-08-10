# Persistence Doc Definition

A **persistence** instance documents one serialization or save-file concern — the on-disk or wire format and its compatibility guarantees. In `tiles_tools`, that means the save file model built around `SaveManager` and `GameStateSerializer`, with its storage model, data layout, and durability guarantees written down. This collection holds one instance per concern; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `tiles_tools`' on-disk save-file formats.
- **Responsibility**: Document each format's storage model, data layout, and durability guarantees.
- **In Scope**: The save file model (`SaveManager`/`GameStateSerializer`).
- **Out of Scope**: The ECS runtime this format has no live bridge to yet (see `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Save File Model](001_save_file_model.md) | 2-file-per-save layout, 3 serialization formats, durability gaps | ✅ |
