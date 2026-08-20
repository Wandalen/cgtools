# 291: Reconsider gpu_hal mipmap/MSAA/compute support if a real consumer emerges

## Execution State

- **id:** 291
- **title:** Reconsider gpu_hal mipmap/MSAA/compute support if a real consumer emerges
- **state:** 📝 (Draft)
- **open:** true
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-18 02:47:12
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/gpu_hal
- **actor:** null
- **started_at:** null
- **expires_at:** null

## MOST Goal

**This is explicitly a tracking placeholder, not active work — do not claim.** `module/helper/gpu_hal`'s
v0 surface (`docs/layer/002_l1_gpu_hal.md § Status`) deliberately stops at the opaque path: buffers, 2d
textures, samplers, shader modules, bind groups, one-color-attachment render passes, a depth attachment,
and texture upload/readback — all proven end-to-end across the webgpu/webgl/native/vulkan backends. Three
capabilities remain explicitly uncovered by design: **mipmaps, MSAA, and compute**. No consumer in this
workspace (`renderer`'s opaque path, `tilemap_renderer`'s d2 adapters, the `triangle_browser` /
`opaque_path_browser` / `adapter_browser` examples) currently needs any of the three — building them now
would be pure speculation with no test surface to validate against, a direct YAGNI violation. No
implementation should begin until a real, concrete consumer need exists.

## In Scope

- **If and only if a real consumer emerges:** design and implement whichever of the three capabilities
  that consumer actually needs (mipmaps, MSAA, or compute — not necessarily all three), following the
  same buy-vs-build/backend-parity discipline already established for the v0 surface (uniform behavior
  across webgpu/webgl/native/vulkan, proven via the same render-and-readback or browser-pixel-verified
  test pattern used for the existing surface).

## Out of Scope

- Any speculative implementation now, absent a concrete consumer — this task exists to keep the door
  open, not to schedule work.
- Partial/half-implemented support for any of the three capabilities on only some backends.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized without a fresh, concrete
  trigger. If revisited, Delivery Requirements should be re-derived at that time against whichever real
  consumer's actual needs, not written speculatively now.

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a watch-item task by design (mirrors task 056's and task 098's
  pattern; see `task/draft/056_vectorizer_revival_watch_item.md` and
  `task/draft/098_obj_viewer_example_proposal_watch_item.md`). Not intended to progress through SUBMIT/
  VERIFY toward 🎯 Verified/claimable state unless a real consumer need first materializes.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 02:47:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:12:11 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Watch-condition re-verified, not submitted: `docs/layer/002_l1_gpu_hal.md § Status` (current) still reads "Not yet covered: mipmaps, MSAA, compute — accepted YAGNI scope boundary, tracked as a watch-item (task 291)" — claim holds. Adversarial sweep for a hidden consumer found `tilemap_renderer`'s `adapter-webgl` has real non-`Off` `MipmapMode` GL wiring (`webgl_helpers.rs:640-645`, `NEAREST_MIPMAP_NEAREST` etc.) — but confirmed that adapter bypasses `gpu_hal` entirely via its own direct `minwebgl` dependency (the docs' own documented "accepted-until-strangled" posture), so it is not a `gpu_hal` consumer and does not contradict this task's claim; worth noting for whoever revisits this task if/when that adapter is ever strangled onto the HAL. No MSAA/compute consumer signal found anywhere outside `gpu_hal`/L0 crates. Correctly remains 📝 Draft — not run through SUBMIT/`.claim_verify`/`.verify_pass`, per this task's own explicit "do not claim" design; mirrors precedent watch-items 056/098 (`task/cancelled/056_vectorizer_revival_watch_item.md`, `task/cancelled/098_obj_viewer_example_proposal_watch_item.md`), both of which stayed in Draft until their own watch conditions independently resolved via CANCEL, never via SUBMIT. |
