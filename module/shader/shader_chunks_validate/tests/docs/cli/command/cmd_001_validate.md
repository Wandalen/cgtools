# Command Test :: validate

Source: [`../../../../docs/cli/command/01_validate.md`](../../../../docs/cli/command/01_validate.md)

### Parameter Edge Tests (PAR-N)

*N/A — `.validate` declares zero parameters (see
[`../../../../docs/cli/command/01_validate.md`](../../../../docs/cli/command/01_validate.md)'s
Parameters table).*

### Parameter Group Corner Tests (GRP-N)

*N/A — `.validate` declares zero parameters, so no
[parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md)
applies to it.*

### Integration Tests (INT-N)

Direct-call and subprocess cases. The 5 checks' own fixture-isolated
behavior is cited from
[`../command_group/01_validate.md`](../command_group/01_validate.md)
instead, per the `_core`-split precedent in
[`shader_chunks_render/tests/docs/cli/command/cmd_001_render.md`](../../../../../shader_chunks_render/tests/docs/cli/command/cmd_001_render.md):

| ID | Case | Real Test |
|----|------|-----------|
| INT-1 | A clean local fixture chunk produces the explicit all-clear message | `shader_chunks_validate/tests/validate_cli_test.rs::clean_fixture_produces_the_all_clear_message` |
| INT-2 | The real bundled registry is reported clean through this crate's own CLI wiring | `shader_chunks_validate/tests/validate_cli_test.rs::the_real_bundled_registry_is_reported_clean_through_the_cli_wiring` |
| INT-3 | A single finding produces a readable `"1 finding(s):"` report and exit code 1 | `shader_chunks_validate/tests/validate_cli_test.rs::one_finding_produces_a_readable_report_with_exit_code_one` |
| INT-4 | Two independent findings are joined as separate, blank-line-separated blocks in one report | `shader_chunks_validate/tests/validate_cli_test.rs::multiple_findings_are_joined_as_separate_blank_line_separated_blocks` |
| INT-5 | The aggregated `shader_chunks` binary reports the bundled registry clean through a real subprocess invocation | `cli_subprocess_test.rs::validate_reports_the_bundled_registry_clean_through_the_aggregated_binary` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 0 (no parameters) |
| GRP-N | 0 (no parameters, so no group applies) |
| INT-N | 5 |

### See Also

- [`../../../../docs/cli/command/01_validate.md`](../../../../docs/cli/command/01_validate.md) — command source
- [`../command_group/01_validate.md`](../command_group/01_validate.md) — group invariants + engine-level citations
- [`plain_text`](../../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md) — output format
