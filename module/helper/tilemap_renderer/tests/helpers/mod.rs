//! Shared test fixtures for the `tilemap_renderer` test suite.
//!
//! Provides helper functions used across multiple test files to avoid
//! duplication and keep individual tests focused on the behavior under test.

use tilemap_renderer::assets::Assets;

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
