//!
//! # Tools for reading and writing embroidery files
//!

#![allow(clippy::unnecessary_unwrap)]

use mod_interface::mod_interface;

mod private {}

mod_interface!
{
  layer embroidery_file;
  layer stitch_instruction;
  layer format;
  layer thread;
  layer metadata;
  layer error;
}
