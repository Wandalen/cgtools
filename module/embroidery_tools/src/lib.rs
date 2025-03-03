//!
//! # Tools for reading and writing embroidery files
//!

use mod_interface::mod_interface;

mod private
{
  pub const READ_SRC: &str = include_str!("../read.py");
  pub const WRITE_SRC: &str = include_str!("../write.py");
}


mod_interface!
{
  own use READ_SRC;
  own use WRITE_SRC;
  layer embroidery_file;
  layer stitch_instruction;
  layer format;
  layer thread;
  layer metadata;
  layer error;
}
