//! A hand-selected chunk set missing a transitive dependency must fail
//! [`shader_chunks_core::dependency_closed`]'s compile-time assert — the
//! set-completeness guarantee the crate readme documents ( "a forgotten
//! import fails this assert at build time" ).

use shader_chunks_core::{ chunk, dependency_closed, ChunkDescriptor };

// `fbm3` depends on `value_noise` ( which depends on `hash21` ), so a set
// of just `fbm3` is not dependency-closed.
const UNCLOSED : &[ ChunkDescriptor ] = &[ chunk( "fbm3" ) ];

const _ : () = assert!( dependency_closed( UNCLOSED ) );

fn main()
{
}
