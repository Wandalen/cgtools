//! A high-performance, extensible tile-based game development library.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

#![cfg_attr( not( feature = "enabled" ), allow( unused ) )]

#[cfg(feature = "enabled")]
pub mod coordinates;

#[cfg(feature = "enabled")]
pub mod collection;

#[cfg(feature = "enabled")]
pub mod geometry;

#[cfg(feature = "enabled")]
pub mod pathfind;

#[cfg(feature = "enabled")]
pub mod layout;
