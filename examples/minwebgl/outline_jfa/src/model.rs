use std::collections::HashMap;

use gltf::*;

pub struct Reader
{
  source : HashMap< String, Gltf >,
}

impl Reader
{
  pub fn new() {}

  pub fn load( gltf_path : AsRef< str > ) {}
}
