use std::{ cell::RefCell, rc::Rc, collections::HashSet };
use minwebgl as gl;
use gl::{ GL, F32x3 };
use renderer::webgl::{ Node, Scene, Material, helpers, material::PbrMaterial, loaders::gltf };
use renderer::webgl::loaders::gltf::GLTF;
use rustc_hash::FxHashMap;
use crate::gem::GemMaterial;


pub type RcScene = Rc< RefCell< Scene > >;
pub type RcNode = Rc< RefCell< Node > >;
pub type RcVec< T > = Rc< RefCell< Vec< T > > >;

#[ derive( Clone ) ]
#[ non_exhaustive ]
pub struct Ring
{
  pub scene : RcScene,
  pub gems : FxHashMap< String, RcNode >,
}

/// Per-ring color selection (stored separately from Ring for lazy loading support)
#[ derive( Clone ) ]
#[ non_exhaustive ]
pub struct RingColors
{
  pub gem : String,
  pub metal : String,
}

impl Default for RingColors
{
  #[ inline ]
  fn default() -> Self
  {
    Self
    {
      gem : "white".to_string(),
      metal : "silver".to_string(),
    }
  }
}

/// Container for all loaded ring scenes and their gem mappings.
#[ non_exhaustive ]
pub struct RingsInfo
{
  /// One scene per ring variant
  pub rings : RcVec< Option< Ring > >,

  /// Per-ring color selections (available even for unloaded rings)
  pub ring_colors : Vec< RingColors >,

  /// Index of the currently selected ring
  pub current_ring : usize,

  loaded_rings : RefCell< HashSet< usize > >,
  ring_loader : Rc< dyn Fn( GLTF, usize ) + 'static >,
  gl : GL
}

impl RingsInfo
{
  #[ inline ]
  pub(crate) fn new< F >
  (
    gl : &GL,
    rings : RcVec< Option< Ring > >,
    ring_colors : Vec< RingColors >,
    current_ring : usize,
    ring_loader : F
  ) -> Self
  where
    F : Fn( GLTF, usize ) + 'static
  {
    RingsInfo
    {
      rings,
      ring_colors,
      current_ring,
      loaded_rings : RefCell::new( HashSet::new() ),
      ring_loader : Rc::new( ring_loader ),
      gl : gl.clone(),
    }
  }

  #[ must_use ]
  #[ inline ]
  pub fn get_ring( &self ) -> Option< Ring >
  {
    if let Some( ring ) = &self.rings.borrow()[ self.current_ring ]
    {
      return Some( ring.clone() );
    }

    // Use atomic insert to prevent race conditions on rapid ring switching
    if self.loaded_rings.borrow_mut().insert( self.current_ring )
    {
      let gl = self.gl.clone();
      let index = self.current_ring;
      let loader = self.ring_loader.clone();

      let future = async move
      {
        let Some( window ) = gl::web_sys::window()
        else
        {
          gl::log::error!( "Failed to get window object" );
          return;
        };
        let Some( document ) = window.document()
        else
        {
          gl::log::error!( "Failed to get document object" );
          return;
        };

        // Handle GLTF load failures gracefully to prevent WASM crashes on network errors
        let Ok( gltf ) = gltf::load( &document, format!( "./gltf/{index}.glb" ).as_str(), &gl ).await
        else
        {
          return;
        };

        ( loader )( gltf, index );
      };

      gl::spawn_local( future );
    }

    None
  }
}

/// Extracts the base color from a material, handling both
/// PBR metal materials and custom gem materials.
#[ must_use ]
#[ inline ]
pub fn get_color( material : &Rc< RefCell< Box< dyn Material > > > ) -> F32x3
{
  let type_name =
  {
    let m = material.borrow();
    m.type_name()
  };

  match type_name
  {
    "PbrMaterial" =>
    {
      let material = helpers::cast_unchecked_material_to_ref::< PbrMaterial >( material.borrow() );
      material.base_color_factor.truncate()
    },
    "GemMaterial" =>
    {
      let material = helpers::cast_unchecked_material_to_ref::< GemMaterial >( material.borrow() );
      material.color
    },
    _ => F32x3::splat( 1.0 )
  }
}

/// Removes numeric characters from a string.
/// Used to group gem instances sharing the same base name.
#[ must_use ]
#[ inline ]
pub fn remove_numbers( s : &str ) -> String
{
  s.chars().filter( | c | !c.is_ascii_digit() ).collect()
}
