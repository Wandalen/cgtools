//! Backend adapter implementations.

#[ cfg( feature = "adapter-svg" ) ]
mod svg;

#[ cfg( feature = "adapter-terminal" ) ]
mod terminal;

#[ cfg( feature = "adapter-webgl" ) ]
mod webgl;
