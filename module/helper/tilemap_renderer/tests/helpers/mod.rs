//! Shared test fixtures for the `tilemap_renderer` test suite.
//!
//! Provides helper functions used across multiple test files to avoid
//! duplication and keep individual tests focused on the behavior under test.

use tilemap_renderer::assets::Assets;

#[ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]
#[ allow( dead_code, reason = "tests/helpers is recompiled per integration-test binary; only the three webgl_*_test.rs browser suites call these, and not each of them every helper, so they read as dead in the others — expect would be unfulfilled where they are used" ) ]
pub mod webgl;

#[ allow( dead_code, reason = "tests/helpers is recompiled per integration-test binary; webgl_pending_image_test.rs / webgl_premultiplied_test.rs pull it in for `webgl` alone, so it reads as dead there — expect would be unfulfilled in every other binary" ) ]
pub fn empty_assets() -> Assets
{
  Assets
  {
    fonts : vec![],
    images : vec![],
    sprites : vec![],
    geometries : vec![],
    gradients : vec![],
    patterns : vec![],
    clip_masks : vec![],
    paths : vec![],
  }
}
