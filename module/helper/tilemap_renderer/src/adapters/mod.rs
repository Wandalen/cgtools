//! Backend adapter implementations.

#[ cfg( feature = "adapter-svg" ) ]
mod svg;
#[ cfg( feature = "adapter-svg" ) ]
pub use svg::SvgBackend;

#[ cfg( feature = "adapter-terminal" ) ]
mod terminal;
#[ cfg( feature = "adapter-terminal" ) ]
pub use terminal::TerminalBackend;

// Future adapters:
// #[ cfg( feature = "adapter-webgl" ) ]
// mod webgl;
// #[ cfg( feature = "adapter-wgpu" ) ]
// mod wgpu_backend;
