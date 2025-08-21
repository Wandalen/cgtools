#![ allow( clippy::implicit_return ) ]

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
  layer helper;
  layer buffer;
  layer context;
  layer texture;
  layer error;
}
