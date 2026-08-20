# Replace `tilemap_renderer`'s `adapter-svg` `image`-crate dependency with a minimal `png`-crate dependency

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** Q-02
- **repo_identity:** self
- **unit_type:** module
- **unit:** module/helper/tilemap_renderer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **verification_date:** 2026-08-11 20:40:20
- **blocked_by:** null
- **executing_at:** 2026-08-11 20:13:24
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **priority:** 0
- **in_motion:** false
- **accepting_at:** 2026-08-11 20:29:27
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **completed_at:** 2026-08-11 20:40:20
- **completed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

Implement the native half of `task/decisions.md`'s Q-02 decision: replace `adapter-svg`'s dependency
on the full multi-format `image` crate with a minimal dependency on the `png` crate (already
resolved in `Cargo.lock` at version 0.17.16 via `image`'s own transitive dependency graph — confirmed
via direct `grep`/`awk` on `Cargo.lock`), narrowing the crate's only two `image`-crate call sites —
`bitmap_to_png` (`svg.rs:508-531`) and `image_dimensions` (`svg.rs:541-548`; confirmed via
`grep "image::" src/ tests/` to be the complete list, with zero other `image::` usage anywhere else
in the crate) — to `png`-crate-backed implementations. `bitmap_to_png` (raw pixel bytes tagged with a
`PixelFormat` — `Rgba8`/`Rgb8`/`Gray8`/`GrayAlpha8`, a crate-shared enum defined in `assets.rs` and
also consumed by `webgl.rs` and `native.rs` — encoded to PNG file bytes for embedding as
`data:image/png;base64,...` inside emitted SVG `<image>` elements) rewrites against `png::Encoder`:
map each `PixelFormat` variant to its `png::ColorType`/`BitDepth::Eight` counterpart
(`Rgba8→ColorType::Rgba`, `Rgb8→ColorType::Rgb`, `Gray8→ColorType::Grayscale`,
`GrayAlpha8→ColorType::GrayscaleAlpha` — `png::ColorType` has an exact counterpart for all 4), then
`Encoder::new(&mut buf, width, height).set_color(..).set_depth(..).write_header()?.write_image_data(bytes)?`,
then an explicit `.finish()?` (the `png` crate's `Writer::finish` performs the real
IEND-write-and-flush; its `Drop` impl silently swallows any error via `let _ = self.write_iend()`, so
relying on `Drop` instead of an explicit `.finish()` call would hide real encode failures).
`write_image_data`'s own internal buffer-length validation against `width*height*bpp` preserves the
existing dimension-mismatch-returns-`None` contract the test suite already covers
(`bitmap_to_png_dimension_mismatch_returns_none`, `svg.rs:2236`) — no separate manual length check is
needed. `image_dimensions` (encoded-file bytes → `(width, height)`, used to size the SVG `<image>`
element without a full pixel decode) rewrites against
`png::Decoder::new(reader).read_header_info()?` — header-only, lighter than a full `.read_info()`
decode — returning `(info.width, info.height)`. This IS a disclosed contract narrowing (unlike
`bitmap_to_png`, which stays fully 4-format-capable on its own raw-pixel-layout axis):
`image_dimensions`'s current doc comment and implementation are genuinely multi-format (PNG/JPEG/
GIF/WebP/BMP/TIFF via `image::ImageReader::with_guessed_format().into_dimensions()`); after this
task, non-PNG `Encoded` bytes degrade to the same `(0,0)`-fallback path the `Encoded` match arm
(`svg.rs:982-1001`) already takes for malformed bytes today (`.unwrap_or((0,0))`), rather than
successfully extracting dimensions from a real non-PNG image. No existing test depends on non-PNG
dimension-extraction succeeding — `tests/svg_backend_test.rs`'s `image_encoded_jpeg_emits_jpeg_mime`
(line 1315) only asserts MIME-string embedding, never dimension correctness. `detect_image_mime`
(`svg.rs:553-561`, a pure magic-byte sniffer with zero `image`-crate dependency) and the test-only
hand-rolled `png_dimensions` IHDR reader (`svg.rs:567-575`, `#[cfg(test)]`-gated, already PNG-only)
are both unaffected. Testable: `cargo nextest run -p tilemap_renderer --features
adapter-svg,adapter-terminal,cli,scene-model` stays green (including all 5 existing
`bitmap_to_png_*` tests across all 4 `PixelFormat` variants and the dimension-mismatch case) and
`cargo tree -p tilemap_renderer --features adapter-svg -e normal` no longer lists `image`.

## In Scope

- Rewrite `bitmap_to_png` (`svg.rs:508-531`) against `png::Encoder`/`png::Writer`, preserving all 4
  `PixelFormat` variant support (`Rgba8`/`Rgb8`/`Gray8`/`GrayAlpha8`) and the
  dimension-mismatch-returns-`None` contract
- Rewrite `image_dimensions` (`svg.rs:541-548`) against `png::Decoder`, header-only read (PNG-only;
  disclosed narrowing from the current multi-format support)
- `Cargo.toml`: `adapter-svg` feature drops `dep:image`, adds `dep:png`; `[dependencies]` drops the
  `image` line, adds `png = { workspace = true, optional = true }`
- Root `Cargo.toml`: add `[workspace.dependencies.png]` `version = "0.17.16"` (matching the version
  already resolved in `Cargo.lock` via `image`'s transitive graph today)
- Doc comment updates: `image_dimensions`'s own doc comment (currently claims PNG/JPEG/GIF/WebP/BMP/
  TIFF support) and the SVG-facing half of `assets.rs`'s `ImageSource::Encoded` doc comment, both
  narrowed to disclose PNG-only dimension extraction
- `roadmap.md`'s design-decisions table row ("Bitmap images encoded to PNG via `image` crate")
  updated to name the `png` crate instead
- New round-trip test: encode via the rewritten `bitmap_to_png`, decode back via `png::Decoder`,
  assert pixel-for-pixel equality against the original input (the existing tests only check
  magic-bytes/`is_some()`, not pixel fidelity)

## Out of Scope

- **`detect_image_mime`** — zero `image`-crate dependency already; untouched by this task regardless
  of whether `task/completed/092_tilemap_renderer_webgl_encoded_image_decode.md` has relocated it to
  `assets.rs` yet
- **The test-only hand-rolled `png_dimensions` IHDR reader** (`svg.rs:567-575`) — already PNG-only,
  `#[cfg(test)]`-gated, unaffected
- **`adapter-webgl`'s own `ImageSource::Encoded` decoding** — that is task 092, deliberately
  independent (browser-native decode, no `png` crate involved at all)
- **Adding non-PNG native decode support for `image_dimensions`** (e.g. a second minimal
  single-format crate for JPEG) — no concrete need demonstrated; the decision text itself names this
  as revisable, not committed
- **Path/text/group rendering, gradient/pattern/clip-mask GPU loading, effects,
  `BlendMode::Overlay`, terminal adapter stub work** — unrelated roadmap gaps in other adapters
- **Live rendering/visual regression testing beyond the new pixel round-trip test** — no visual-diff
  tooling exists in this workspace

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not by this
section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model`
    passes with zero failures; `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model -- -D warnings`
    exits 0
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model` after the `png`-crate rewrite | native, full non-webgl feature set | 0 failures — all 5 existing `bitmap_to_png_*` tests (all 4 `PixelFormat` variants + dimension-mismatch) and existing dimension/MIME tests stay green |
| T02 | `cargo tree -p tilemap_renderer --features adapter-svg -e normal \| grep -c '^image '` | native, `adapter-svg` only | `0` — confirms the `image` crate is no longer linked |
| T03 | `cargo check -p tilemap_renderer --no-default-features --features adapter-terminal` | native, `adapter-terminal` only | Exit 0 — regression: terminal-only builds unaffected by the shared `PixelFormat` enum's encoder-side change |
| T04 | `git diff --stat -- src/adapters/webgl.rs` after the change | — | This task adds zero further hunks to `webgl.rs` beyond whatever task 092 already left uncommitted (this repo has no per-task commits, so the raw diff-from-HEAD is not itself empty — it must match exactly task 092's own already-isolated contribution, documented in `task/completed/092_tilemap_renderer_webgl_encoded_image_decode.md`'s `## Outcomes` C1/C2/C5-C7/C9 line references) — confirms no encroachment into task 092's domain |
| T05 | New test: encode a non-trivial `Rgba8`/`Rgb8`/`Gray8`/`GrayAlpha8` buffer via `bitmap_to_png`, decode it back via `png::Decoder`, compare pixels | native, `adapter-svg` | Decoded pixels equal the original input exactly, for all 4 `PixelFormat` variants — the honest correctness bar the existing magic-byte/`is_some()` checks don't cover |

## Acceptance Criteria

-   `bitmap_to_png` produces valid PNG bytes for all 4 `PixelFormat` variants (`Rgba8`/`Rgb8`/
    `Gray8`/`GrayAlpha8`), decodable by `png::Decoder` with pixel data matching the original raw
    input (round-trip fidelity)
-   `bitmap_to_png` still returns `None` on a dimension/byte-count mismatch (existing
    `bitmap_to_png_dimension_mismatch_returns_none` test, `svg.rs:2236`, continues to pass)
-   `image_dimensions` returns correct `(width, height)` for PNG input via
    `png::Decoder::read_header_info` (header-only, no full pixel decode)
-   `Cargo.toml`'s `adapter-svg` feature no longer references `dep:image`; `cargo tree` confirms
    `image` is not linked when building with only `adapter-svg`
-   Root `Cargo.toml` declares `[workspace.dependencies.png]` at version `0.17.16`
-   `png::Writer::finish()` is called explicitly (not relied on via `Drop`) so encode failures
    surface as `Result::Err`, not silently swallowed
-   Non-PNG `Encoded` bytes degrade to the existing `(0,0)` dimension fallback — `svg.rs`'s
    `Encoded` match arm's `<defs>` embedding still proceeds per its pre-existing graceful-degrade
    behavior
-   `image_dimensions`'s doc comment and `assets.rs`'s `ImageSource::Encoded` doc comment both
    disclose the PNG-only narrowing
-   Every row T01–T05 passes
-   `git diff --stat -- src/adapters/webgl.rs` is empty

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Encoder rewrite**
- [x] C1 — Does `bitmap_to_png` use `png::Encoder`/`Writer` (not the `image` crate) to produce PNG
      bytes, with all 4 `PixelFormat` variants mapped to their `png::ColorType` counterparts?
- [x] C2 — Is `Writer::finish()` called explicitly and its `Result` propagated (not left to `Drop`)?
- [x] C3 — Does the dimension-mismatch case (undersized buffer) still return `None`, not panic or
      silently produce corrupt PNG data?

**Decoder rewrite**
- [x] C4 — Does `image_dimensions` use `png::Decoder::read_header_info` (header-only) rather than a
      full pixel decode?
- [x] C5 — Does non-PNG input degrade to the pre-existing `(0,0)`-fallback path, not a panic or a new
      error type?

**Dependency swap**
- [x] C6 — Does `Cargo.toml`'s `adapter-svg` feature list drop `dep:image` and add `dep:png`?
- [x] C7 — Does the root `Cargo.toml` declare `png` at version `0.17.16` under
      `[workspace.dependencies]`?
- [x] C8 — Does `cargo tree` confirm `image` is no longer linked when building with only
      `adapter-svg`?

**Out of Scope confirmation**
- [x] C9 — Are `detect_image_mime` and the test-only `png_dimensions` IHDR reader untouched by this
      task?
- [x] C10 — Is `src/adapters/webgl.rs` untouched by this task's diff (task 092's domain)?
- [x] C11 — Does this task's diff leave every other `roadmap.md` adapter gap (path/text/group
      rendering, gradient/pattern/clip-mask GPU loading, effects, `BlendMode::Overlay`, terminal
      adapter stub work) untouched — no partial implementation of any of them?
- [x] C12 — Does the Test Matrix add only the pixel round-trip test (T05) beyond existing
      regression — no new visual-diff/rendering-comparison tooling?

**Documentation**
- [x] C13 — Do `image_dimensions`'s doc comment and `assets.rs`'s `ImageSource::Encoded` doc comment
      both disclose the PNG-only dimension-extraction narrowing?

### Measurements

- [x] M1 — `git diff --stat -- Cargo.toml` (root): expect the new `[workspace.dependencies.png]`
      table added
- [x] M2 — `git diff --stat -- src/adapters/webgl.rs`: expect empty

### Invariants

- [x] I1 — `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model` → 0 failures
- [x] I2 — `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model -- -D warnings` → 0 warnings
- [x] I3 — `cargo tree -p tilemap_renderer --features adapter-svg -e normal | grep -c '^image '` → 0

### Anti-faking checks

- [x] AF1 — The round-trip test (T05) genuinely decodes the encoder's own output and compares
      pixels — a test that only checks the output byte length or a PNG magic-byte header (as
      today's existing tests do) would still pass while silently encoding garbage pixel data
- [x] AF2 — `Writer::finish()`'s `Result` is genuinely propagated with `?`/`.ok()?` (or equivalent),
      not discarded — a discarded result would still compile and pass I1-I3 while silently
      swallowing real encode failures, identical to the crate's own `Drop`-path behavior this task
      deliberately avoids. (Disclosed gap: no test forces `write_image_data`/`finish` itself to
      fail — doing so would require a broken `Write` sink, disproportionate to this task's scope —
      so this check is code-inspection-based, not failure-injection-tested.)

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — design-decisions table, "Bitmap images encoded to
  PNG via `image` crate" row (updated as part of this task's own scope)
- `task/decisions.md` — Q-02, the decision this task implements the native half of
- `module/helper/tilemap_renderer/src/adapters/svg.rs:508-575` (`bitmap_to_png`,
  `image_dimensions`, `detect_image_mime`, `png_dimensions` — all 4 functions this task's scope
  boundary runs through)
- `module/helper/tilemap_renderer/src/assets.rs:414-480` (`ImageSource`/`PixelFormat` enums and doc
  comments)
- `module/helper/tilemap_renderer/tests/svg_backend_test.rs:1315`
  (`image_encoded_jpeg_emits_jpeg_mime` — confirms no existing test depends on non-PNG dimension
  extraction)
- `module/helper/tilemap_renderer/src/adapters/svg.rs:2198-2241` (existing `bitmap_to_png_*` test
  block — regression bar for the encoder rewrite)
- `task/completed/092_tilemap_renderer_webgl_encoded_image_decode.md` — sibling task implementing the
  web half of the same Q-02 decision

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-11 20:13:24 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_EXEC | execution claimed |

## History

- **[2026-08-11]** `FILED` — Task filed by user1@w002 via `doc_tsk`. Goal: implement the native half
  of `task/decisions.md`'s Q-02 decision (minimal single-format `png`-crate dependency for
  `adapter-svg`, replacing the full multi-format `image` crate).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Confirmed via `grep -rn "image::" src/ tests/` that all `image`-crate usage in the entire crate is confined to `svg.rs:512-543` — no hidden usage elsewhere; this task's 2-function rewrite is a complete removal, not partial | — |
| D2 | MOST Goal Quality | — | 🟡→🟢 | Adversarial: initial goal draft (before re-reading source) assumed `bitmap_to_png` was RGBA8-only; direct read of `svg.rs:510-524` revealed a 4-variant `PixelFormat` match (`Rgba8`/`Rgb8`/`Gray8`/`GrayAlpha8`, shared with `webgl.rs`/`native.rs`) already covered by 5 existing tests (`svg.rs:2200-2241`) | Goal, In Scope, Test Matrix, and Acceptance Criteria all revised to require preserving all 4 variants (confirmed `png::ColorType` has an exact counterpart for each), not just the RGBA8 case |
| D3 | Value / YAGNI | — | 🟢 | Directly implements the user's own just-made Q-02 decision; declines to add non-PNG native decode support, matching the decision's own "revisable if needed" framing rather than pre-building it | — |
| D4 | Implementation Readiness | — | 🟡→🟢 | Adversarial: `png` crate API independently verified via direct source read (`Encoder`/`Writer`/`Decoder`/`ColorType`/`BitDepth`); version 0.17.16 confirmed already resolved in `Cargo.lock`. But does any test exercise `Writer::finish()`'s error path? No — only the success path is tested; forcing a real encode failure would need a broken `Write` sink, disproportionate to this task | Disclosed as an accepted, non-blocking gap in AF2 (code-inspection-based guarantee, not failure-injection-tested) rather than silently claimed as fully covered |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | `png` is already present in the workspace's resolved dependency graph (transitively, via `image`) — adding it as a direct workspace dependency stays within this crate's and the root `Cargo.toml`'s existing dependency-declaration scope | — |
| D8 | Crate Single Responsibility | — | 🟢 | PNG encoding/dimension-extraction is already this crate's established responsibility (raster-to-SVG embedding); swapping the backing implementation adds no new responsibility | — |
| **Total** | | — | 🟢 | 0 open | 2 fixes |

**Verified by:** self (Tier 2 Dual-Role Self-Check) · **Date:** 2026-08-11
| 2026-08-11 20:27:10 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | EXEC_COMPLETE | execution complete |
| 2026-08-11 20:29:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_ACCEPT | acceptance claimed |

## Outcomes

### Acceptance Results

- **Verified by:** independent verifier session, dispatched with zero access to the executing
  session/context (per `tsk.rulebook.md § Acceptance Verification : Procedure - Execution`'s
  Separation of Concerns). Resolved actor identity: `user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/`.
- **Independence note:** this resolved actor identity is byte-identical to the task's `executing_by`
  field. This is a known, disclosed limitation (BUG-197), not a sign of self-verification: `scope
  get::id` is deterministic per host+user+cwd, not per-session, so it collides for any verifier
  running on this machine regardless of session isolation. Disclosed per instruction rather than
  hidden; the walk below was performed from scratch with no access to the executing session's
  context, reasoning, or conclusions — all evidence below was independently gathered via fresh file
  reads and fresh command executions in this session.
- **Date:** 2026-08-11
- **Verdict:** PASS

#### Checklist

- [x] C1 — Does `bitmap_to_png` use `png::Encoder`/`Writer` (not the `image` crate) to produce PNG
      bytes, with all 4 `PixelFormat` variants mapped to their `png::ColorType` counterparts? — YES:
      `src/adapters/svg.rs:511-534` — `color_type` match (511-519) maps `Rgba8→Rgba`, `Rgb8→Rgb`,
      `Gray8→Grayscale`, `GrayAlpha8→GrayscaleAlpha`; body uses `png::Encoder::new(&mut png_bytes,
      width, height)`, `.set_color(..)`, `.set_depth(png::BitDepth::Eight)`, `.write_header()?`,
      `.write_image_data(bytes)?`. `git diff` confirms `use image::DynamicImage` and all
      `DynamicImage::Image*8`/`from_raw` arms were removed, not left dormant alongside.
- [x] C2 — Is `Writer::finish()` called explicitly and its `Result` propagated (not left to `Drop`)? —
      YES: `svg.rs:532` — `writer.finish().ok()?;`, with an inline comment explaining `Drop` alone
      would silently swallow encode errors.
- [x] C3 — Does the dimension-mismatch case (undersized buffer) still return `None`, not panic or
      silently produce corrupt PNG data? — YES: `bitmap_to_png_dimension_mismatch_returns_none`
      (`svg.rs:2224-2229`) passed in the live `cargo nextest run` (see I1); relies on
      `write_image_data(bytes).ok()?` (`svg.rs:529`) returning `Err` (not panicking) when the byte
      count doesn't match `width*height*bpp`.
- [x] C4 — Does `image_dimensions` use `png::Decoder::read_header_info` (header-only) rather than a
      full pixel decode? — YES: `svg.rs:546-547` — `png::Decoder::new(std::io::Cursor::new(bytes))`
      then `.read_header_info().ok()?`; no `.read_info()`/full-frame-decode call anywhere in the
      function.
- [x] C5 — Does non-PNG input degrade to the pre-existing `(0,0)`-fallback path, not a panic or a new
      error type? — YES: malformed/non-PNG bytes make `png::Decoder::read_header_info` return `Err`,
      which `image_dimensions` converts to `None` via `.ok()?`; the sole call site
      (`svg.rs:975`) retains `Self::image_dimensions(bytes).unwrap_or((0,0))`, unchanged.
- [x] C6 — Does `Cargo.toml`'s `adapter-svg` feature list drop `dep:image` and add `dep:png`? — YES:
      `module/helper/tilemap_renderer/Cargo.toml:17` —
      `adapter-svg = ["enabled", "dep:base64", "dep:bytemuck", "dep:png"]`; `git diff` confirms
      `-dep:image`/`+dep:png` in the feature list and `-image = { workspace = true, optional = true }`
      / `+png = { workspace = true, optional = true }` in `[dependencies]`.
- [x] C7 — Does the root `Cargo.toml` declare `png` at version `0.17.16` under
      `[workspace.dependencies]`? — YES: root `Cargo.toml:351-352` — `[workspace.dependencies.png]` /
      `version = "0.17.16"`; `git diff -- Cargo.toml` shows exactly this 3-line addition (see M1).
- [x] C8 — Does `cargo tree` confirm `image` is no longer linked when building with only
      `adapter-svg`? — YES: `cargo tree -p tilemap_renderer --features adapter-svg -e normal | grep -c
      '^image '` → `0` (executed live); full tree inspected — `png v0.17.16` present with its own
      sub-dependencies (`crc32fast`, `fdeflate`, `flate2`, `miniz_oxide`, `bitflags`), `image` absent
      entirely.
- [x] C9 — Are `detect_image_mime` and the test-only `png_dimensions` IHDR reader untouched by this
      task? — YES: `detect_image_mime`'s 6-branch magic-byte logic is byte-for-byte identical
      before/after — it was relocated from a `SvgBackend::`-scoped private method in `svg.rs` to a
      `pub(crate) fn` in `assets.rs:557-565` by sibling task 092 (cross-checked against task 092's own
      Checklist C3/C4 evidence, `task/completed/092_..._webgl_encoded_image_decode.md` lines 273-274);
      the removed body in the `svg.rs` diff and the added body in the `assets.rs` diff match exactly,
      confirming a pure relocation with no logic change. `png_dimensions` (`svg.rs:555-563`,
      `#[cfg(test)]`-gated) appears in the diff only as unmodified surrounding context — zero edits to
      its own lines.
- [x] C10 — Is `src/adapters/webgl.rs` untouched by this task's diff (task 092's domain)? — YES:
      `git diff --stat -- src/adapters/webgl.rs` → 130 insertions/83 deletions (not literally empty —
      expected, since task 092's own uncommitted contribution coexists in this working tree; this repo
      has no per-task commits — see M2); `git diff -- src/adapters/webgl.rs | grep -E "^[+-]" | grep
      -icE "bitmap_to_png|image_dimensions|png::|dep:png|use png"` → `0`. Full diff read (5 hunks)
      confirms every change — the `ImageSource::Encoded` blob-decode rewrite, the extracted
      `upload_bitmap_texture` helper, and the `on_load`/`on_error` blob-URL revocation guards — falls
      within the line ranges task 092's own Outcomes cites as its isolated contribution (webgl.rs:
      828-844, 851, 864, 1420-1422, 1477-1479, 1489). Nothing unaccounted for.
- [x] C11 — Does this task's diff leave every other `roadmap.md` adapter gap (path/text/group
      rendering, gradient/pattern/clip-mask GPU loading, effects, `BlendMode::Overlay`, terminal
      adapter stub work) untouched — no partial implementation of any of them? — YES: `git diff --
      roadmap.md` shows exactly 3 changed lines: the `ImageSource::Encoded` bullet (task 092's
      contribution — references Q-02/task 092), the "WebGL context loss handling" bullet removal
      (task 090's contribution, per task 092's own Outcomes note), and the design-decisions table row
      "Bitmap images encoded to PNG via `png` crate ..." (this task's own, sole contribution to this
      file). Every other adapter-gap bullet is byte-identical, unchanged.
- [x] C12 — Does the Test Matrix add only the pixel round-trip test (T05) beyond existing regression —
      no new visual-diff/rendering-comparison tooling? — YES: full `svg.rs` diff (5 hunks) reviewed —
      exactly one new test added, `bitmap_to_png_round_trip_pixel_fidelity` (`svg.rs:2235-2257`); all
      other hunks are the encoder/decoder rewrite, one import line, and `detect_image_mime` call-site
      renames (092's relocation). Live `cargo nextest run` shows 123 total tests (up from 122
      documented in task 092's own I2 evidence), consistent with exactly one net new test.
- [x] C13 — Do `image_dimensions`'s doc comment and `assets.rs`'s `ImageSource::Encoded` doc comment
      both disclose the PNG-only dimension-extraction narrowing? — YES: `image_dimensions`'s own `///`
      doc comment (`svg.rs:536-541`) states "PNG-only: unlike the crate's previous `image`-backed
      implementation, non-PNG bytes ... no longer resolve dimensions"; `assets.rs`'s `ImageSource::
      Encoded` doc comment (`assets.rs:437-439`) states "Dimensions are extracted via a minimal
      PNG-only header read (the `png` crate). Non-PNG bytes ... do not resolve dimensions."

#### Measurements

- [x] M1 — `git diff --stat -- Cargo.toml` (root): `Cargo.toml | 3 +++` — MET (expected: the new
      `[workspace.dependencies.png]` table added). Exact content confirmed via `git diff`:
      `+[workspace.dependencies.png]` / `+version = "0.17.16"` / `+` (blank line) — nothing else
      changed in the root `Cargo.toml`.
- [x] M2 — `git diff --stat -- src/adapters/webgl.rs`: `130 insertions(+), 83 deletions(-)` — literal
      reading of "expect empty" is MISSED, but this exact non-emptiness is explicitly anticipated and
      disambiguated by this same task file's own Test Matrix row T04 ("this repo has no per-task
      commits, so the raw diff-from-HEAD is not itself empty ... it must match exactly task 092's own
      already-isolated contribution") and by the Acceptance Criteria's own C10 pairing. Applying that
      same-file disambiguation (not an outside charitable reinterpretation): the domain-scoped
      sub-check — zero lines touching `bitmap_to_png`/`image_dimensions`/`png::`/`dep:png`/`use png` —
      returns `0` (see C10), and every hunk in the diff is independently attributable to task 092. On
      that basis: **MET** in the sense the task file itself defines "empty" to mean in this
      no-per-task-commit repo. Flagged as a **non-blocking task-authoring wording defect**: the
      Acceptance Criteria bullet "`git diff --stat -- src/adapters/webgl.rs` is empty" and M2's own
      "expect empty" phrasing are literally inaccurate in this repo's structure and should read
      "empty of this task's own domain" to match T04's own more careful wording — does not change the
      verdict since the substantive claim (zero encroachment) is independently verified true via C10.

#### Invariants

- [x] I1 — `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model`
      → HOLD: `123 tests run: 123 passed, 0 skipped`, exit 0, elapsed 12s
      (`module/helper/tilemap_renderer/-0043_longrun.log`). Includes all 5 pre-existing
      `bitmap_to_png_*` tests plus the new `bitmap_to_png_round_trip_pixel_fidelity`.
- [x] I2 — `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --features
      adapter-svg,adapter-terminal,cli,scene-model -- -D warnings` → HOLD: exit 0, 0 warnings. First
      pass (`-0044_longrun.log`) returned a cargo-fingerprint cache hit with no visible "Checking"
      line (same pattern task 092's own I3 disclosed) — `src/adapters/svg.rs` and `src/assets.rs` were
      touched (mtime only, no content change) to force a genuine re-lint; the forced re-run
      (`-0045_longrun.log`) shows an explicit "Checking tilemap_renderer v0.2.0" pass, still exit 0,
      0 warnings.
- [x] I3 — `cargo tree -p tilemap_renderer --features adapter-svg -e normal | grep -c '^image '` →
      HOLD: `0`. Cross-confirmed via full tree read: `png v0.17.16` present, `image` absent.

#### Anti-faking checks

- [x] AF1 — The round-trip test (T05) genuinely decodes the encoder's own output and compares pixels —
      PASS: `svg.rs:2235-2257` (`bitmap_to_png_round_trip_pixel_fidelity`) decodes via
      `png::Decoder::new(..)` / `reader.read_info()` / `reader.next_frame(&mut buf)`, then
      `assert_eq!(&buf[..info.buffer_size()], pixels, "pixel mismatch for {format:?}")` for all 4
      `PixelFormat` variants with non-trivial, non-uniform pixel values (e.g.
      `[10,20,30,40,50,60,70,80,90,100,110,120,130,140,150,160]` for `Rgba8`) — genuine byte-level
      pixel comparison, not an `is_some()`/magic-byte-only check. Confirmed passing in the live
      `cargo nextest run` (I1, `-0043_longrun.log` line 16).
- [x] AF2 — `Writer::finish()`'s `Result` is genuinely propagated with `?`/`.ok()?` (or equivalent),
      not discarded — PASS: `svg.rs:532` — `writer.finish().ok()?;` converts `Result<(),
      EncodingError>` to `Option` and early-returns `None` via `?` on `Err`; not a bare
      `let _ = writer.finish();`. The disclosed gap (no failure-injection test forces `finish()`
      itself to error) stands as documented in the task's own AF2 text — this remains a
      code-inspection-based guarantee, consistent with the task's own stated scope.

**Additional non-blocking observation (not a Verification-layer item, does not affect verdict):**
`svg.rs:983-985`'s inline comment inside `load_assets`'s `ImageSource::Encoded` match arm still reads
"Decode dimensions for any format the `image` crate recognizes (PNG, JPEG, GIF, WebP, ...)". This
predates this task (byte-identical in `HEAD`, confirmed via `git show HEAD:.../svg.rs | grep`) and is
not one of the two doc comments this task's own Acceptance Criteria names for the PNG-only-narrowing
disclosure (C13 covers only `image_dimensions`'s own doc comment and `assets.rs`'s
`ImageSource::Encoded` doc comment, both of which are correctly updated). It is, however, now
factually stale given this task's own implementation change (the function it describes is PNG-only as
of this task), sitting two lines above the very call site (`svg.rs:975`) whose behavior this task
narrowed. Recommended for a follow-up documentation touch-up; does not fail any Checklist item as
written and does not affect this PASS verdict.

**Structural note (disclosed per instruction, not a project finding):** task 092
(`task/accepting/092_tilemap_renderer_webgl_encoded_image_decode.md`), the sibling task implementing
the web half of the same Q-02 decision, shows an identical pattern in its own file: a complete,
independently-verified `Verdict: PASS` Outcomes section, while its Execution State `state` field still
reads `🔎 (Accepting)` rather than `✅ (Completed)`. This is consistent with the same `tsk
.acceptance_pass` self-verification guard (BUG-197) this verifier expects to hit in Step 8 below —
included here only as corroborating evidence that the collision is a systemic tool limitation on this
host, not specific to this task or this verifier's own identity resolution.
| 2026-08-11 20:40:20 | task | ACCEPTANCE_PASS | acceptance passed |
