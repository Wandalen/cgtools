# Parameter :: 23. serve

- **Fundamental Type:** [`Switch`](../../../../shader_chunks_query/docs/cli/type/07_switch.md) (unilang
  `Kind::Boolean`)
- **Constraints:** `1`/`true`/`yes` and `0`/`false`/`no` (unilang's
  boolean coercion); anything else is rejected by unilang before the
  command routine runs
- **Default:** `true`
- **Purpose:** Controls whether `.preview` hands off to the browser dev
  server after building, validating, and writing the bundle. `true`
  (default) blocks the process on `action/browser_serve` until the
  server stops; `false` returns immediately after printing the summary,
  which is what makes the command scriptable and testable via subprocess
  without a browser.

### Examples
```bash
# Valid values
preview fbm3                 # serve::1 is the default — builds, then blocks serving in the browser
preview fbm3 serve::1        # same, explicit
preview fbm3 serve::0        # builds, validates, writes, prints the summary, exits — no browser

# Invalid values (rejected with error)
preview fbm3 serve::maybe    # unilang boolean coercion failure, non-zero exit
```

### Notes
- `serve::0` never skips validation — a chunk that fails naga
  parse/validation still exits 1 regardless of `serve::`'s value, since
  validation happens in `bundle_prepare`, before the bundle is even
  written.
- This is the only `Switch` parameter in the CLI whose default is `true`
  rather than `false` — every other boolean (`case`, `transitive`,
  `roots`, `leaves`, `count`) defaults off; `serve` defaults on because
  the command's whole purpose is "show it live," and `serve::0` is the
  deliberate opt-out for scripted/CI use.
- Member of no [parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md) — it has no
  filtering/projection/formatting role; it selects whether a side effect
  (the dev-server hand-off) happens at all.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.preview](../command/01_preview.md) | `true` | Only command with a `Switch` parameter defaulting on |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [Switch](../../../../shader_chunks_query/docs/cli/type/07_switch.md) | Boolean | `bool` | `1/true/yes` vs `0/false/no` |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
