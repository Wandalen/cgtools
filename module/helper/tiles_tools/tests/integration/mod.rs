//! Integration tests for tiles_tools
//!
//! All integration tests are feature-gated with the "integration" feature
//! to allow for selective test execution.

#![cfg(feature = "integration")]
#![allow(clippy::needless_return)]
#![allow(clippy::implicit_return)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::similar_names)]
#![allow(clippy::duplicated_attributes)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_else)]

mod conversion_tests;
mod coordinates_tests;
mod ecs_tests;
// mod field_of_view_tests;
// Temporarily disabled until flowfield generic constraints are resolved
// mod flowfield_tests;
mod isometric_coords_tests;
mod square_coords_tests;
mod triangular_coords_tests;
// mod grid_tests;
// mod pathfinding_tests;
// mod generation_tests;
