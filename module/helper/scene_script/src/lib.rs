#![ doc( html_root_url = "https://docs.rs/scene_script/latest/scene_script/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Rhai scripting glue for describing and animating 2D scenes." ) ]

mod private
{
}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// `F32x2` Rhai type and operator registration.
  layer vector_binding;

  /// `animation::Tween< F32x2 >` Rhai type registration.
  layer tween_binding;

  /// Pre-configured `rhai::Engine` builder.
  layer engine;
}
