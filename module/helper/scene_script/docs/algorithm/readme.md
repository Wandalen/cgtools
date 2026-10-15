# Algorithm Doc Definition

An **algorithm** documents a HOW — a step-by-step computational procedure with correctness properties worth stating explicitly. In `scene_script`, this collection is the navigational hub for the one algorithmic procedure the crate implements: classifying a compiled script's top-level statements so the declarative-bindings convention can be enforced. This collection holds one instance per algorithm; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s step-by-step computational procedures.
- **Responsibility**: Document each algorithm's abstract behavior and concrete step-by-step procedure.
- **In Scope**: The top-level statement classification procedure `check_top_level_is_declarative()` runs.
- **Out of Scope**: The property this procedure enforces (see `invariant/001`, which states the WHAT this algorithm's HOW maintains).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Top-Level Statement Classification](001_top_level_statement_classification.md) | Classify each top-level statement into Binding/PlainExpression/Call/Imperative so the checker can allow or reject it by role and position | ✅ |
