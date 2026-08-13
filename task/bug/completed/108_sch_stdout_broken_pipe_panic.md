# BUG-108: `sch`/`shader_chunks` panics on a closed stdout pipe — violating the crate's own "never a panic" contract

- **Severity:** Medium
- **state:** Completed
- **Affects:** Every piped invocation whose reader stops before the CLI finishes writing — `sch list | head -1`, `sch compose ... | grep`, any `| head`/`| true`-shaped pipeline
- **Component:** `module/shader/shader_chunks` (`src/main.rs` stdout/stderr printing; entry point since dissolved into `src/cli.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-13
- **filed_by:** user1@w002
- **verified_by:** user1@w002
- **verification_date:** 2026-08-13
- **Fixed:** 2026-08-13

## Symptom

```
$ sch list | true
thread 'main' (733950) panicked at library/std/src/io/stdio.rs:1166:9:
failed printing to stdout: Broken pipe (os error 32)
$ echo "${PIPESTATUS[0]}"
101
```

The crate's `readme.md` promises every failure "exits non-zero with a
message on stderr — never a panic". A closed pipe produces a raw panic
backtrace and exit 101.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
sch list | true ; echo "exit=${PIPESTATUS[0]} (expect quiet exit 0, not 101 with a panic)"
```
**Expected:** the CLI ends quietly when its reader closes the pipe — conventional Unix behavior, exit 0, no stderr noise.
**Actual:** `thread 'main' panicked ... failed printing to stdout: Broken pipe`, exit 101.

## Impact

**Who is affected:** Every shell user composing the CLI into a pipeline —
piping into `head`, `grep -m1`, or any early-exiting consumer is the normal
way to use a listing tool.

**What breaks:** All stdout printing goes through the `println!` macro
(`print_help`, `print_command_help`, and the central `result.outputs` loop
in `main`), and error reporting through `eprintln!` — both macros panic on
any write error, including `EPIPE`. Rust's `Stdout` is always line-buffered,
so the panic fires deterministically on the first write after the reader
closes; it escaped the test suite only because every existing subprocess
test reads the child's output to completion, and small outputs fit the pipe
buffer when the reader races ahead.

## How Discovered

Intermittent `failed printing to stdout: Broken pipe` panic in a reinstall
smoke-check log (`sch compose help | head -3`); made deterministic by
closing the pipe's read end before the process writes (`sch list | true`).

## Fix

`src/cli.rs` (`Fix(BUG-108)` comment at the site):

1. All stdout writes route through one `print_stdout` choke point — `writeln!` to the locked handle; `ErrorKind::BrokenPipe` maps to a quiet `exit( 0 )` (the Unix convention for a reader that hung up); any other write failure reports on stderr (best-effort) and exits 2. The three former `println!` sites (`print_help`, `print_command_help`, the central `result.outputs` loop) all use it.
2. All stderr writes route through `print_stderr`, which discards write errors — error reporting can never itself become a second panic path; the mapped `CliError` exit code survives a closed stderr.
3. Reproducers close the read end of a real OS pipe (`std::io::pipe`) BEFORE spawning the binary, so the child's first write hits `EPIPE` with zero race — `closed_stdout_pipe_ends_quietly_without_a_panic` (expects exit 0, no backtrace) and `closed_stderr_pipe_still_exits_with_the_mapped_code` (expects exit 1, not a panic's 101), both marked `bug_reproducer(BUG-108)` in `tests/cli_subprocess_test.rs`.

**Verification:** RED confirmed pre-fix (both reproducers failed: panic, exit 101). Post-fix `verb/test_only pkg::shader_chunks` — 68/68 passed; `cargo clippy -p shader_chunks --all-targets --all-features -- -D warnings` — exit 0. Reinstalled binary probed: `sch list | true` → exit 0, no stderr output (was: panic backtrace, exit 101).
