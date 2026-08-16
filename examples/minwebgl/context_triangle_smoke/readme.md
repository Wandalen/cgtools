# Context Triangle Smoke Test

**Keywords:** Tutorial, Basics, WebGL2, Context Creation, Pixel Verification, Smoke Test

The narrowest possible browser-side proof that `minwebgl::context::from_canvas` actually
works: create a canvas, get a WebGL2 context from it, compile and link one shader pair,
upload one triangle's vertices, and draw. This is the browser-side pixel-verification
counterpart to `minwebgl`'s native `tests/` suite (which covers only the pure-logic layer)
— `from_canvas` and everything downstream of it cannot be exercised without a real GL
context, so this crate exists to give that path at least one real, `browsee`-verified
data point instead of zero.

**[How to run](../../how_to_run.md)**
