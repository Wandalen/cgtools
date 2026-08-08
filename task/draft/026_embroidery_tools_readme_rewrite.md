# Rewrite embroidery_tools/readme.md to match the real API

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/embroidery_tools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`module/helper/embroidery_tools/readme.md` (236 lines, read in full this session) documents an entirely
fictional API — `EmbroideryPattern`, `pattern.stitch_count()`/`.color_count()`/`.add_color()`/`.scale()`/
`.rotate()`/`.optimize()`, `Stitch::normal()`, `Color::rgb()`, `PesVersion::V6` — none of which exist in
the real source. The actual API (confirmed by grepping `src/embroidery_file.rs`, `thread.rs`,
`format/pes.rs`, `format/pec.rs` this session) is `EmbroideryFile` with `new()`, `stitches()`, `threads()`,
`stitch(dx,dy)`, `jump(dx,dy)`, `color_change(dx,dy)`, `trim()`, `end()`, `add_stitch_relative/absolute()`,
`add_thread()`, `bounds()`, `as_command_blocks()`; `Color`, `Thread`; `PESVersion` (not `PesVersion`);
`pec_threads() -> [Thread; 65]`. P4 (rewrite bucket) — rewrite the readme's Quick Start and API Reference
sections entirely from the real API surface; the "Current Status & Roadmap" / "Use Cases" prose sections
may be salvageable if reworded to stop implying the fictional API works.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P4 (doc rewrite)
  tier, Rewrite-from-scratch bucket.
