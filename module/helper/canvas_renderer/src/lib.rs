#![ doc( html_root_url = "https://docs.rs/canvas_renderer/latest/canvas_renderer/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renderer for opaque 2D objects" ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  layer renderer;
}