<!-- bug_system_metadata
type: integrated
version: 1.0
-->

# bug

Bug reports for the cgtools workspace. IDs share the tsk Unified ID namespace tracked in
`../readme.md` (`highest_id`).

## Responsibility Table

| File | Responsibility |
|------|-----------------|
| readme.md | Bug index and open bugs tracking |
| draft/ | Newly filed bugs, structurally incomplete |
| unverified/ | Structurally complete bugs awaiting the VERIFY gate |
| verifying/ | Bugs actively undergoing the VERIFY gate |
| verified/ | Bugs confirmed and claimable for fix work |
| executing/ | Bugs with an in-progress fix |
| executed/ | Bugs whose fix landed, awaiting acceptance review |
| accepting/ | Bugs under acceptance review |
| completed/ | Bugs whose fix is verified and closed |
| cancelled/ | Bugs closed as won't-fix or duplicate |
| mixed/ | Bugs with cross-boundary or entirely-foreign fix scope |
| orphan/ | Mixed bugs confirmed for full external relocation |

## Open Bugs

| ID | Title | State | Severity | Component | Filed | Root Cause | Round |
|----|-------|-------|----------|-----------|-------|------------|-------|
| BUG-007 | [csgrs's mandatory core2 dependency is permanently yanked](./verified/007_csgrs_core2_yanked_dependency.md) | Verified | Critical | workspace root Cargo.toml | 2026-08-08 | core2 ^0.4 (csgrs's mandatory dep) is entirely yanked | 1 |
