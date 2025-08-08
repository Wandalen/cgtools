//! This crate provides a comprehensive toolkit for working with tile-based maps,
//! with a special focus on hexagonal grids. It offers modules for handling various
//! coordinate systems, managing collections of tiles in grid structures, defining
//! layouts, handling geometric calculations, and performing pathfinding.

#![cfg_attr(not(feature = "enabled"), allow(unused))]

pub mod coordinates;

#[cfg(feature = "enabled")]
pub mod collection;

#[cfg(feature = "enabled")]
pub mod geometry;

#[cfg(feature = "enabled")]
pub mod pathfind;

#[cfg(feature = "enabled")]
pub mod layout;
