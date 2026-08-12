# BUG-103: `sch help` succeeds silently and `sch compose help` misparses `help` as a chunk name — every help invocation is silent or wrong

- **Severity:** Medium
- **state:** Draft
- **Affects:** Every conventional help invocation of the `shader_chunks`/`sch` CLI — the only working spelling is the bare, argument-less binary name
- **Component:** `module/shader/shader_chunks` (`src/main.rs` dispatch)
- **repo_identity:** self
- **Filed:** 2026-08-13
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/

## Symptom

```
$ sch help
$ echo $?
0                                    # exit 0, zero bytes of output

$ sch compose help
Execution error: Execution Error: unknown chunk: `help` (see `list` for valid names)
```

`sch .`, `sch .help`, `sch compose ??`, and `sch .compose.help` — all of
`unilang`'s own help spellings — likewise exit 0 with zero bytes of output.
Only bare `sch` prints usage.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo run -q -p shader_chunks --bin sch -- help ; echo "exit=$? (expect usage text above, not silence)"
cargo run -q -p shader_chunks --bin sch -- compose help ; echo "exit=$? (expect compose help, not 'unknown chunk')"
```
**Expected:** top-level usage for the first, per-command help for the second, both exit 0.
**Actual:** first prints nothing (exit 0 — silent success); second exits 1 treating `help` as a chunk name.

## Impact

**Who is affected:** Every CLI user typing any conventional help spelling; `help` silently "succeeding" is a silent failure of exactly the discovery path new users try first.

**What breaks:** Two coupled defects in `src/main.rs`:
1. `main` never prints `result.outputs` — each command routine `println!`s its own success content, so every framework-generated output (the `.` listing, `.help`, `?`/`??` per-command help, the auto-registered `.{command}.help` builtins) is computed, returned, and dropped.
2. No mapping exists from the conventional spellings (`help`, `<command> help`) to those framework help forms — bare `help` dot-normalizes onto the builtin `.help` whose output defect 1 swallows, and a trailing `help` binds as an ordinary positional argument (`compose help` → chunk lookup).

## How Discovered

User-reported: `sch help` at the terminal printed nothing after installing via `verb/install/run`, then `sch compose help` errored with `unknown chunk: help`. Reproduced against a fresh build; all five framework help spellings confirmed silent via exit-code/output probe.
