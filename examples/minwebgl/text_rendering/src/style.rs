//! Style modifiers applied on top of already-built glyph meshes : synthetic
//! italic ( vertex shear ), synthetic bold ( a second, slightly-enlarged pass
//! sharing the glyph's own material ), and an underline quad. None of these
//! need a bold/italic font face — the repository has none — so each is the
//! standard synthetic approximation used when a true weight/slant face is
//! unavailable.

use crate::{ AttributesData, PrimitiveData, Transform };
use renderer::webgl::Material;
use std::cell::RefCell;
use std::rc::Rc;

/// Applies a synthetic italic shear ( `x += factor * y` ) to every primitive's
/// own vertex positions. Deep-clones each primitive's attributes first —
/// glyph meshes are cached per character and shared via `Rc` across every
/// occurrence of that character, so mutating in place would shear the font's
/// cached geometry itself, not just this one piece of text.
pub fn mesh_shear_x( mesh : &mut [ PrimitiveData ], factor : f32 )
{
  for primitive in mesh.iter_mut()
  {
    let sheared = primitive.attributes.borrow().positions.iter()
    .map( | [ x, y, z ] | [ x + factor * y, *y, *z ] )
    .collect();
    let normals = primitive.attributes.borrow().normals.clone();
    let indices = primitive.attributes.borrow().indices.clone();

    primitive.attributes = Rc::new( RefCell::new( AttributesData { positions : sheared, normals, indices } ) );
  }
}

/// Adds a synthetic bold pass : for every primitive, an extra copy scaled up
/// by `growth` about its own local origin, sharing the same material, pushed
/// alongside the original so the glyph silhouette reads visually thicker.
pub fn mesh_bold_apply( mesh : &mut Vec< PrimitiveData >, growth : f32 )
{
  let extra : Vec< PrimitiveData > = mesh.iter().map( | primitive |
  {
    let positions = primitive.attributes.borrow().positions.iter()
    .map( | [ x, y, z ] | [ x * growth, y * growth, *z ] )
    .collect();
    let normals = primitive.attributes.borrow().normals.clone();
    let indices = primitive.attributes.borrow().indices.clone();

    PrimitiveData
    {
      attributes : Rc::new( RefCell::new( AttributesData { positions, normals, indices } ) ),
      material : primitive.material.clone(),
      transform : primitive.transform.clone()
    }
  })
  .collect();

  mesh.extend( extra );
}

/// Builds a thin quad primitive spanning `width`, its top edge at local
/// `y = 0` — position via `.transform` at a text's baseline to underline it.
#[ must_use ]
pub fn underline_quad_make( width : f32, thickness : f32, material : Rc< RefCell< Box< dyn Material > > > ) -> PrimitiveData
{
  let hw = width / 2.0;
  let positions = vec!
  [
    [ -hw, 0.0, 0.0 ],
    [ hw, 0.0, 0.0 ],
    [ hw, -thickness, 0.0 ],
    [ -hw, -thickness, 0.0 ],
  ];
  let normals = vec![ [ 0.0, 0.0, 1.0 ]; 4 ];
  let indices = vec![ 0, 1, 2, 0, 2, 3 ];

  PrimitiveData
  {
    attributes : Rc::new( RefCell::new( AttributesData { positions, normals, indices } ) ),
    material,
    transform : Transform::default()
  }
}
