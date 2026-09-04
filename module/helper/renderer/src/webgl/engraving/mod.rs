//! Dynamic text engraving: mapping glTF nodes to engraving zones ([`config`]), rasterizing
//! user-supplied text onto an offscreen canvas ([`canvas_text`]), uploading it to a persistent
//! GPU texture without reallocation ([`texture`]), and wiring all of that into a scene's
//! [`crate::webgl::material::PbrMaterial`] instances by node name ([`apply`]).
//!
//! On the shader side this pairs with the `USE_ENGRAVING` block in `shaders/main.frag`: a
//! derivative-based bevel/normal perturbation at mask edges, plus a roughness/albedo change
//! inside the mask, simulating a laser-etched groove. See `engraving_config.schema.json` and
//! `engraving_config.example.json` next to this file for the on-disk config format.

mod private
{

}

crate::mod_interface!
{
  /// `engraving_config.json` schema types and parsing.
  layer config;

  /// Font loading + canvas text rasterization.
  layer canvas_text;

  /// Persistent, in-place-updatable GPU mask texture.
  layer texture;

  /// Node-name matching: binds a config to a loaded scene's materials.
  layer apply;
}
