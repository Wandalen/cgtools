//! Verifies, via source-inspection ( see `Fix(BUG-290)`'s precedent,
//! `minvulkan/tests/context_test.rs`, and this session's own `Fix(BUG-424)`/`Fix(BUG-425)`
//! follow-ups ), that `context::from_canvas_with` keeps a WebGL2 context's viewport in sync
//! with its canvas's drawing-buffer size both initially and across every subsequent
//! CSS-driven resize (BUG-423). Genuinely unreachable from a native `cargo test` run ( no JS
//! engine, no live `ResizeObserver` ) ; a live-browser pixel reproduction was attempted this
//! session ( `examples/minwebgl/context_triangle_smoke` via `browsee`/Firefox ) and found
//! structurally uninformative -- that example draws exactly once with no render loop, and per
//! the HTML canvas spec any resize clears the drawing buffer regardless of whether the viewport
//! fix is present, so both the fixed and broken states render identically blank after a resize.
//! A Chromium-based alternative ( reading `gl.getParameter(gl.VIEWPORT)` live via CDP `.eval`,
//! sidestepping the redraw confound entirely ) was also attempted and blocked by a pre-existing,
//! unrelated Chromium WebGL failure in this environment ( `"Uncaught RuntimeError: unreachable"` ),
//! not by anything under this fix's control. See `task/bug/completed/423_*.md` for the full
//! verification record.

// test_kind: bug_reproducer(BUG-423)
/// ## Root Cause
/// `mingl::web::canvas`'s `ResizeObserver` ( the one `canvas::make()` attaches ) keeps a
/// canvas's `width`/`height` attributes synced to its CSS box, but never calls `gl.viewport(..)`
/// -- it is deliberately GL-unaware, shared substrate reused by `minwebgl::canvas`,
/// `minwebgpu::canvas`, and `minwebgl::texture::d2::sprite_upload`'s own temporary 2D-context
/// canvas. Any WebGL2 context already bound to that canvas therefore kept rendering into its
/// *original* viewport rectangle after any CSS-driven resize ( window resize, flex reflow,
/// devtools docking, ... ), clipping or stretching the visible image into a stale rectangle even
/// though the underlying drawing buffer itself had correctly resized.
///
/// ## Why Not Caught
/// `from_canvas_with` needs a live `WebGl2RenderingContext` bound to a real `HtmlCanvasElement`
/// inside an actual browser layout engine, neither constructible from a native `cargo test` run.
/// Live-browser pixel verification was attempted this session against
/// `examples/minwebgl/context_triangle_smoke` ( the only example crate covering
/// `context::from_canvas` + a draw call ) via `browsee`/Firefox : the initial render confirmed
/// correct ( centered red triangle ), and a scripted window resize was confirmed to actually
/// apply ( `532x412` -> `900x700` via a follow-up `.windows` query ), but the post-resize
/// screenshot came back blank white in *both* the fixed and hypothetically-broken states --
/// the example calls `app_run()` exactly once with no `requestAnimationFrame` loop, and per the
/// HTML canvas spec, resizing a canvas's `width`/`height` attribute always clears its drawing
/// buffer regardless of context type, so nothing redraws after the resize either way. A
/// Chromium-based alternative ( reading `gl.getParameter(gl.VIEWPORT)` directly via CDP `.eval`,
/// which does not depend on a redraw ) was also attempted, but blocked by a pre-existing,
/// unrelated Chromium WebGL failure in this environment ( wasm `"Uncaught RuntimeError:
/// unreachable"` on load, reproducing independently of any change made here ). Modifying or
/// adding an `examples/` crate to build a render-loop-bearing test page was out of reach : it
/// is outside this fix's edit scope. Source-inspection is therefore the same fallback already
/// established this workspace-wide for defects that are real but structurally unreachable from
/// the available native/in-scope test surface.
///
/// ## Fix Applied
/// `from_canvas_with` now calls `gl.viewport(..)` once immediately after context creation ( so
/// the very first frame already matches the buffer, not just after the first resize ), and
/// attaches a second, GL-aware `ResizeObserver` -- independent of `mingl::web::canvas`'s -- whose
/// callback calls `mingl::web::canvas::canvas_resize` ( now `pub`, exposing the exact same
/// width/height computation `canvas::make()`'s own observer uses ) and then `gl.viewport(..)`
/// with the freshly recomputed size, so the buffer and the viewport can never disagree.
///
/// ## Prevention
/// This test asserts the exact fixed source text is present : the initial `viewport` call, and
/// the second `ResizeObserver`'s callback body ( both its `canvas::canvas_resize` call and its
/// `viewport` re-application ). A regression that dropped either the initial call or the second
/// observer would fail this test. The fix's cross-crate precondition -- `canvas_resize` staying
/// `pub` in `mingl` -- is deliberately not re-checked here : `mod_interface!`'s own generated
/// `own use canvas_resize;` re-export already requires at least `pub` visibility to compile at
/// all, confirmed empirically this session by temporarily downgrading it to both private and
/// `pub( crate )` and observing `mingl` itself fail to build in each case ( `E0432`/`E0364` )
/// before this test could ever run ; a separate source-inspection assertion for that specific
/// precondition would be unreachable-in-failure and therefore pure duplication.
///
/// ## Pitfall
/// A canvas's drawing-buffer size ( `width`/`height` attributes ) and a bound WebGL context's
/// viewport are two independent pieces of state -- resizing one never implies resizing the
/// other, and a `ResizeObserver` that only updates the former ( as shared, GL-unaware canvas
/// substrate correctly does ) will silently desync any GL viewport bound to that canvas unless
/// something GL-aware explicitly re-applies it on every resize, not just at context creation.
#[ test ]
fn from_canvas_with_syncs_viewport_initially_and_on_every_resize()
{
  let context_src = include_str!( "../src/context.rs" );

  let initial_viewport_count = context_src
  .matches( "gl.viewport( 0, 0, canvas.width() as i32, canvas.height() as i32 );" )
  .count();
  assert_eq!
  (
    initial_viewport_count, 1,
    "from_canvas_with (BUG-423) must set the viewport once immediately after context creation, \
    found {initial_viewport_count} occurrences of the expected initial viewport call"
  );

  let resize_call_count = context_src.matches( "canvas::canvas_resize( &canvas_clone );" ).count();
  assert_eq!
  (
    resize_call_count, 1,
    "from_canvas_with (BUG-423) must recompute the canvas size via canvas::canvas_resize inside \
    its own ResizeObserver callback, found {resize_call_count} occurrences"
  );

  let resize_viewport_count = context_src
  .matches( "gl_clone.viewport( 0, 0, canvas_clone.width() as i32, canvas_clone.height() as i32 );" )
  .count();
  assert_eq!
  (
    resize_viewport_count, 1,
    "from_canvas_with (BUG-423) must re-apply the viewport from the freshly resized canvas \
    inside its own ResizeObserver callback, found {resize_viewport_count} occurrences"
  );

  let observer_count = context_src.matches( "web_sys::ResizeObserver::new(" ).count();
  assert_eq!
  (
    observer_count, 1,
    "from_canvas_with (BUG-423) must attach its own GL-aware ResizeObserver, found \
    {observer_count} occurrences of ResizeObserver construction in context.rs"
  );
}
