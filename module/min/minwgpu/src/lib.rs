
//!
//! # `minwgpu`
//!
//! Minwgpu is a minimal, opinionated toolkit designed to simplify common
//! `wgpu` patterns. It provides convenient builders and helpers to reduce
//! boilerplate when setting up a `wgpu` context, managing buffers, and more,
//! making it easier to get a graphics application up and running.
//!

use mingl::mod_interface;

mod private {}

mod_interface!
{
  // Re-export the underlying host API, so a consumer reaches `wgpu` through this driver
  // rather than naming it as a second, independently-versioned dependency of its own —
  // matching `minwebgl` and `minwebgpu`, which each re-export `web_sys` the same way.
  own use ::wgpu;

  layer helper;
  layer buffer;
  layer context;
  layer texture;
  layer surface;
  layer bind;
  layer pipeline;
  layer pass;
  layer readback;
  layer error;
}
