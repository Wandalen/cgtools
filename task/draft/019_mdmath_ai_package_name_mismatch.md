# Fix mdmath_ai/mdmath_ia package name transposition

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
- **unit:** lib/yrd_gamedev/cgtools/module/blank/mdmath_ai
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`module/blank/mdmath_ai/Cargo.toml` declares `name = "mdmath_ia"` — a letter-transposed mismatch against
its own directory name `mdmath_ai` (confirmed by direct read this session) — P3 (dead-code/identity
cleanup bucket, Fix-in-place). Rename the package to `mdmath_ai` to match the directory, and grep the
workspace for any `mdmath_ia` references (root `Cargo.toml` dependency declarations, other crates'
`Cargo.toml` files, doc mentions) that need updating in the same change so nothing silently breaks.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code /
  identity cleanup) tier, Fix-in-place bucket.
