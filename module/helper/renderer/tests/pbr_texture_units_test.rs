//! `PbrMaterial`'s texture-unit layout: every sampler the PBR program can bind gets its own
//! unit, and the fragment-stage samplers fit WebGL2's guaranteed minimum.
//!
//! Pure constant checks, no GL context: the layout is the `PBR_*_UNIT` / `PBR_IBL_*`
//! constants in `material/pbr.rs` plus the skinning / morph slots in `skeleton.rs`, and a
//! new texture that reused a unit would silently sample the wrong image.

use renderer::webgl::{ DISPLACEMENTS_SLOT, GLOBAL_MATRICES_SLOT, INVERSE_MATRICES_SLOT };
use renderer::webgl::material::{ PBR_IBL_BASE_UNIT, PBR_IBL_UNIT_COUNT, PBR_TEXTURE_UNITS };

/// `MAX_TEXTURE_IMAGE_UNITS` every WebGL2 implementation guarantees for the fragment stage.
const WEBGL2_MIN_FRAGMENT_SAMPLERS : usize = 16;

fn all_units() -> Vec< ( String, u32 ) >
{
  let mut units : Vec< ( String, u32 ) > = PBR_TEXTURE_UNITS.iter().map( | ( name, unit ) | ( ( *name ).to_owned(), *unit ) ).collect();
  units.push( ( "globalMatrices (skeleton)".to_owned(), GLOBAL_MATRICES_SLOT ) );
  units.push( ( "inverseMatrices (skeleton)".to_owned(), INVERSE_MATRICES_SLOT ) );
  units.push( ( "displacements (skeleton)".to_owned(), DISPLACEMENTS_SLOT ) );
  for i in 0..PBR_IBL_UNIT_COUNT
  {
    units.push( ( format!( "IBL #{i}" ), PBR_IBL_BASE_UNIT + i ) );
  }
  units
}

#[ test ]
fn pbr_texture_units_are_disjoint()
{
  let units = all_units();
  for ( i, ( name_a, unit_a ) ) in units.iter().enumerate()
  {
    for ( name_b, unit_b ) in &units[ i + 1.. ]
    {
      assert_ne!( unit_a, unit_b, "{name_a} and {name_b} share texture unit {unit_a}" );
    }
  }
}

#[ test ]
fn pbr_fragment_samplers_fit_webgl2_minimum()
{
  // The skinning / morph textures are sampled in the vertex stage; the fragment stage sees
  // every material sampler plus the IBL set.
  let fragment_samplers = PBR_TEXTURE_UNITS.len() + PBR_IBL_UNIT_COUNT as usize;
  assert!
  (
    fragment_samplers <= WEBGL2_MIN_FRAGMENT_SAMPLERS,
    "{fragment_samplers} fragment samplers exceed WebGL2's guaranteed {WEBGL2_MIN_FRAGMENT_SAMPLERS}"
  );
}

#[ test ]
fn pbr_ibl_units_follow_skinning_slots()
{
  let last_skinning_slot = GLOBAL_MATRICES_SLOT.max( INVERSE_MATRICES_SLOT ).max( DISPLACEMENTS_SLOT );
  assert!( PBR_IBL_BASE_UNIT > last_skinning_slot, "IBL units must start after the skinning / morph slots" );
}
