use super::*;
use the_module::post_processing::{ ALL, GBufferAttachment };

/// Every [`GBufferAttachment`] must map to a distinct `#define` name — the fragment shader
/// branches on these via `#ifdef`, so a wrong or colliding mapping would silently merge two
/// attachments' shader code paths.
#[ test ]
fn define_const_maps_every_attachment_to_its_own_shader_define()
{
  let expected =
  [
    ( GBufferAttachment::Position, "POSITION" ),
    ( GBufferAttachment::Color, "COLOR" ),
    ( GBufferAttachment::Uv1, "UV_1" ),
    ( GBufferAttachment::Albedo, "ALBEDO" ),
    ( GBufferAttachment::Normal, "NORMAL" ),
    ( GBufferAttachment::PbrInfo, "PBR_INFO" ),
    ( GBufferAttachment::ObjectColor, "OBJECT_COLOR" ),
  ];

  for ( attachment, define ) in expected
  {
    assert_eq!
    (
      attachment.define_const(), define,
      "{attachment:?} must map to the #define name {define:?}"
    );
  }
}

#[ test ]
fn define_const_names_are_unique_across_all_attachments()
{
  let mut names : Vec< String > = ALL.iter().map( | a | a.define_const() ).collect();
  let before = names.len();
  names.sort();
  names.dedup();
  assert_eq!( names.len(), before, "two GBufferAttachment variants share the same #define name" );
}

/// `GBuffer::new` passes each attachment's already-allocated vertex buffers in; the early-return
/// guard on an empty slice avoids a `.expect()` panic for attachments ( e.g.
/// [`GBufferAttachment::Position`] ) that do expect a buffer but were supplied none.
#[ test ]
fn attribute_info_returns_empty_for_every_attachment_when_no_buffers_are_supplied()
{
  for attachment in ALL
  {
    let info = attachment.attribute_info( &[] );
    assert!
    (
      info.is_empty(),
      "{attachment:?}.attribute_info( &[] ) must return an empty Vec, got {} entries", info.len()
    );
  }
}
