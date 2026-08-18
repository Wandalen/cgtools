# Procedure

Entity-lifecycle operations for `docs/cli/`, scoped to the entity types
this CLI actually uses. Each operation is a rulebook Entity Operations
procedure in `cli_doc_des.rulebook.md` — followed exactly, never restated
here.

### Applicable Operations

| Entity | Add | Remove |
|--------|-----|--------|
| Command | `cli_doc_des.rulebook.md § Entity Operations : Add Command · OC055` | `§ Entity Operations : Remove Command · OC060` |
| Parameter | `§ Entity Operations : Add Parameter · OC056` | `§ Entity Operations : Remove Parameter · OC061` |
| Type | `§ Entity Operations : Add Type · OC057` | `§ Entity Operations : Remove Type · OC062` |
| Parameter Group | `§ Entity Operations : Add Parameter Group · OC058` | `§ Entity Operations : Remove Parameter Group · OC063` |
| Format | `§ Entity Operations : Add Format · OC059` | `§ Entity Operations : Remove Format · OC064` |
| Command Group | `§ Entity Operations : Add Command Group · OC163` | `§ Entity Operations : Remove Command Group · OC164` |

When adding a command, parameter, type, or format, follow the target
operation's own step list in full — it already cross-references the other
entity types (e.g. Add Command's own steps 2-3 cover registering new
parameters/types it introduces). A new command additionally registers in
exactly one `command_group/` instance (the partition is complete and
non-overlapping); a new named query parameter registers in exactly one
`param_group/` instance and in *both* query commands' Parameters tables —
`list` and `get` share one `query_arguments` declaration in
`shader_chunks_query/src/lib.rs`, so a parameter cannot exist on only one
of them.

### Inapplicable Operations

These entity types are deliberately absent from this CLI (see
[`readme.md` § Scope Decisions](readme.md#scope-decisions)) — their
Add/Remove operations do not apply until the underlying condition changes:

- **User Story** (`OC065`/`OC066`) — this CLI has only 9 commands total,
  not enough distinct user stories to warrant the collection.
- **Command Noun** (`OC069`/`OC070`) — only one domain noun (`chunk`)
  exists; the collection requires ≥3.
- **Command Verb** (`OC071`/`OC072`) — same reasoning as Command Noun.
- **Environment Parameter / Config Parameter** — this CLI reads zero
  environment variables and zero config file parameters at runtime (pure
  argv); both are conditional L3 blockers that do not apply here.
  `shader_chunks_preview` resolves two paths via compile-time
  `env!("CARGO_MANIFEST_DIR")`, but that is a build-time constant baked
  into the binary, not a runtime environment read, so it does not
  constitute an Environment Parameter in the L3 sense.

*(Parameter Group moved from this list to Applicable Operations when
`list`/`get` unified behind the shared query engine — they now co-occur
on 20 named parameters.)*

If any of these conditions changes (e.g. a second domain noun is
introduced), re-evaluate the corresponding Scope Decision before applying
the entity's Add operation for the first time.
