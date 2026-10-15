//! A typo'd name given to [`shader_chunks_core::chunk`] in `const` position
//! must fail the build — the selective-import guarantee the crate readme
//! documents ( "a typo'd name fails the build" ).

const BROKEN : shader_chunks_core::ChunkDescriptor = shader_chunks_core::chunk( "hash12" );

fn main()
{
  let _ = BROKEN;
}
