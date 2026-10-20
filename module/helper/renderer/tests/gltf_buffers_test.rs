//! Off-GPU checks for the glTF loader's buffer stage : `cpu_buffers_needed`, which decides
//! whether `gltf::load` copies every buffer into wasm memory.

use renderer::webgl::loaders::gltf_buffers::internal::cpu_buffers_needed;

/// Parses `json` without validation, as `gltf::load` does before `document_validate`.
fn parse( json : &str ) -> gltf::Gltf
{
  gltf::Gltf::from_slice_without_validation( json.as_bytes() ).expect( "the fixture is well-formed JSON" )
}

/// One triangle mesh, with `extra` spliced into the document and `targets` into its
/// primitive. Neither has to be valid beyond what the `gltf` crate iterates.
fn document( extra : &str, targets : &str ) -> gltf::Gltf
{
  parse( &format!
  (
    r#"{{
      "asset" : {{ "version" : "2.0" }},
      "buffers" : [ {{ "byteLength" : 36, "uri" : "a.bin" }} ],
      "bufferViews" : [ {{ "buffer" : 0, "byteLength" : 36 }} ],
      "accessors" : [ {{ "bufferView" : 0, "componentType" : 5126, "count" : 3, "type" : "VEC3" }} ],
      "meshes" : [ {{ "primitives" : [ {{ "attributes" : {{ "POSITION" : 0 }} {targets} }} ] }} ],
      "nodes" : [ {{ "mesh" : 0 }} ]
      {extra}
    }}"#
  ) )
}

/// A static mesh is read only by the GPU, so its buffers are not copied into wasm memory :
/// the copy would cost as much as the model again, for nothing.
#[ test ]
fn cpu_buffers_needed_is_false_for_a_static_mesh()
{
  assert!( !cpu_buffers_needed( &document( "", "" ) ) );
}

/// A skin's inverse bind matrices are read on the CPU.
#[ test ]
fn cpu_buffers_needed_for_a_skin()
{
  assert!( cpu_buffers_needed( &document( r#", "skins" : [ { "joints" : [ 0 ] } ]"#, "" ) ) );
}

/// Animation keyframes are read on the CPU.
#[ test ]
fn cpu_buffers_needed_for_an_animation()
{
  let animation = r#", "animations" : [ {
    "channels" : [ { "sampler" : 0, "target" : { "node" : 0, "path" : "translation" } } ],
    "samplers" : [ { "input" : 0, "output" : 0 } ]
  } ]"#;
  assert!( cpu_buffers_needed( &document( animation, "" ) ) );
}

/// Morph target displacements are read on the CPU.
#[ test ]
fn cpu_buffers_needed_for_a_morph_target()
{
  assert!( cpu_buffers_needed( &document( "", r#", "targets" : [ { "POSITION" : 0 } ]"# ) ) );
}
