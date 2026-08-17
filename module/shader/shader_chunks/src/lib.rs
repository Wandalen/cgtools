//! `shader_chunks` ( short alias `sch` ) aggregates the whole
//! `shader_chunks` utility family into one CLI: [`shader_chunks_query`]'s
//! `list`/`get`/`tags`/`tree`, [`shader_chunks_compose`]'s `compose`,
//! [`shader_chunks_params`]'s `tunables`, [`shader_chunks_preview`]'s
//! `preview`, [`shader_chunks_render`]'s `render`, and
//! [`shader_chunks_validate`]'s `validate` — each utility also ships its
//! own standalone binary with byte-identical behavior for its own
//! commands. This crate's only responsibility is aggregation:
//! concatenating each utility's command set, help groups, and help
//! examples ( query, then compose, then params, then preview, then render,
//! then validate — the order every help screen and aggregation test pins )
//! and handing the result to [`shader_chunks_cli_core::run`]. All command
//! logic, argument wiring, and rendering live in the utility crates
//! themselves.

mod private
{
  /// Standalone entry point for the `shader_chunks`/`sch` binaries.
  pub fn run()
  {
    let binary = "shader_chunks";
    let mut groups = Vec::new();
    let mut examples = Vec::new();
    let mut commands = Vec::new();

    groups.extend( shader_chunks_query::help_groups() );
    examples.extend( shader_chunks_query::help_examples( binary ) );
    commands.extend( shader_chunks_query::commands( binary ) );

    groups.extend( shader_chunks_compose::help_groups() );
    examples.extend( shader_chunks_compose::help_examples( binary ) );
    commands.extend( shader_chunks_compose::commands( binary ) );

    groups.extend( shader_chunks_params::help_groups() );
    examples.extend( shader_chunks_params::help_examples( binary ) );
    commands.extend( shader_chunks_params::commands( binary ) );

    groups.extend( shader_chunks_preview::help_groups() );
    examples.extend( shader_chunks_preview::help_examples( binary ) );
    commands.extend( shader_chunks_preview::commands( binary ) );

    groups.extend( shader_chunks_render::help_groups() );
    examples.extend( shader_chunks_render::help_examples( binary ) );
    commands.extend( shader_chunks_render::commands( binary ) );

    groups.extend( shader_chunks_validate::help_groups() );
    examples.extend( shader_chunks_validate::help_examples( binary ) );
    commands.extend( shader_chunks_validate::commands( binary ) );

    shader_chunks_cli_core::run( shader_chunks_cli_core::CliApp
    {
      binary : binary.to_string(),
      tagline : "Inspect, compose, preview, render, and validate shader_chunks_core's bundled WGSL chunks.".to_string(),
      groups,
      examples,
      commands,
    });
  }
}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;
  own use run;
}
