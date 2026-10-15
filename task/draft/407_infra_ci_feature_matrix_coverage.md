# 407: infra — CI feature-matrix coverage

## Execution State

- **id:** 407
- **title:** infra — CI feature-matrix coverage
- **state:** 📝 (Draft)
- **open:** true
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-19 22:51:09
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** repository
- **unit:** lib/yrd_gamedev/cgtools
- **actor:** null
- **started_at:** null
- **expires_at:** null

## MOST Goal

**Tracking placeholder — needs scoping before becoming claimable.** `roadmap.md`'s infra section lists
CI feature-matrix coverage as remaining work: today's verification (`--all-features` builds/tests)
conflates every backend/adapter feature flag combination into one build, which can mask
feature-combination-specific regressions (e.g. a crate that compiles fine with `adapter-webgl +
adapter-svg` together but breaks with `adapter-webgl` alone). No CI configuration exercises the actual
cross-product of backend features (webgpu/webgl/native/vulkan × adapter-* flags). Too large for one-pass
implementation: needs a feature-combination enumeration strategy (exhaustive is likely combinatorially
infeasible — needs a meaningful-subset policy) before any CI YAML changes.

## In Scope

- Enumerate the meaningful feature-flag combinations worth CI coverage (not necessarily exhaustive) for
  `gpu_hal` and `tilemap_renderer`, the two crates with the most backend/adapter feature flags.
- Wire the resulting matrix into the existing `.github/workflows/` CI configuration.

## Out of Scope

- Exhaustive combinatorial coverage of every crate's every feature flag — scope to the highest-value
  subset first.
- The wasm-pack browser test runner itself — separate sibling draft task; this task assumes it exists
  before adding it to the CI matrix.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed out into
  a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements re-derived against
  the actual scoped feature-combination policy at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution. Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — infra remaining-work section
- `.github/workflows/pages.yml` — existing CI workflow precedent (task 203) this would extend

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 22:51:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition spot-checked, not submitted: both citations confirmed accurate — task 203 exists (`task/verifying/203_orrery_flexible_backend_selectable_example.md`), and `.github/workflows/pages.yml` plus `ci.yml` both confirmed present on disk. Correctly remains 📝 Draft; not run through SUBMIT/`.claim_verify`/`.verify_pass`, mirroring precedent watch-items 056/098/291. |
