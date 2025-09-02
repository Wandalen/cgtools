//!
//! This crate provides a high-level abstraction for handling user input
//! in a web browser environment (via WebAssembly). It simplifies the process of
//! capturing keyboard, mouse, and scroll wheel events, managing their state,
//! and providing a clear event-driven or state-polling API for games and applications.
//!
//! The core of the library is the `Input` struct, which, once created, attaches the necessary
//! browser event listeners. It then populates an event queue that can be processed
//! each frame. It also maintains an internal state to answer queries like
//! `is_key_down` or `is_button_down`. Listeners are automatically cleaned up when the
//! `Input` struct goes out of scope.
//!

#![allow(clippy::exhaustive_enums)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::implicit_return)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::string_to_string)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::doc_markdown)]

mod input;
mod util;
pub mod keyboard;
pub mod mouse;

pub use input::*;
pub use util::*;
