use super::the_module;
use the_module::Texture;
use minwebgl as gl;

/// A view built with the `Former` builder and no `.target( … )` must default to
/// `TEXTURE_2D`, as the field doc says: `bind()` passes `target` straight to
/// `gl.bind_texture`, where 0 is `INVALID_ENUM` and leaves the unit's previous
/// texture bound.
#[ test ]
fn former_target_defaults_to_texture_2d()
{
  let texture = Texture::former().end();
  assert_eq!( texture.target, gl::TEXTURE_2D );
  assert!( !texture.is_owning(), "a builder-made texture is a view" );
}

/// `Texture::default()` agrees with the builder.
#[ test ]
fn default_target_is_texture_2d()
{
  let texture = Texture::default();
  assert_eq!( texture.target, gl::TEXTURE_2D );
  assert!( !texture.is_owning(), "a default texture is a view" );
}
