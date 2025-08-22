//! Scene management and command queue system.
//!
//! This module provides the core Scene container that holds and manages
//! ordered collections of rendering commands for a single frame.

#[ cfg( feature = "enabled" ) ]
mod private
{

  // Allow certain clippy warnings for scene management structures
  #![ allow( clippy::exhaustive_structs ) ]
  #![ allow( clippy::missing_inline_in_public_items ) ]
  #![ allow( clippy::implicit_return ) ]
  #![ allow( clippy::must_use_candidate ) ]
  #![ allow( clippy::iter_without_into_iter ) ]

  use crate::commands::RenderCommand;

  /// Core scene container for a single frame (FR-A1).
  ///
  /// A Scene represents everything to be rendered in a single frame,
  /// composed of an ordered list of `RenderCommands` (FR-A2).
  #[ derive( Debug, Clone, PartialEq ) ]
  #[ cfg_attr( feature = "serde", derive( serde::Serialize, serde::Deserialize ) ) ]
  pub struct Scene
  {
    /// Ordered list of render commands for this scene.
    commands : Vec< RenderCommand >,
    /// Optional scene identifier for debugging/tracking.
    id : Option< String >,
  }

  /// Iterator over render commands in a scene.
  pub struct SceneCommandIter< 'a >
  {
    commands : core::slice::Iter< 'a, RenderCommand >,
  }

  /// Mutable iterator over render commands in a scene.
  pub struct SceneCommandIterMut< 'a >
  {
    commands : core::slice::IterMut< 'a, RenderCommand >,
  }

  /// Query results for scene command filtering.
  #[ derive( Debug, Clone ) ]
  pub struct QueryResult< 'a >
  {
    commands : Vec< &'a RenderCommand >,
  }

  impl Scene
  {
    /// Creates a new empty scene.
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        commands: Vec::new(),
        id: None,
      }
    }

    /// Creates a new scene with the given identifier.
    #[ must_use ]
    pub fn with_id( id: impl Into< String > ) -> Self
    {
      Self
      {
        commands: Vec::new(),
        id: Some( id.into() ),
      }
    }

    /// Adds a render command to the scene (FR-A3).
    pub fn add( &mut self, command: RenderCommand )
    {
      self.commands.push( command );
    }

    /// Adds multiple render commands to the scene.
    pub fn add_many< I >( &mut self, commands: I )
    where
      I: IntoIterator< Item = RenderCommand >,
    {
      self.commands.extend( commands );
    }

    /// Returns the number of commands in the scene.
    #[ must_use ]
    pub fn len( &self ) -> usize
    {
      self.commands.len()
    }

    /// Returns an iterator over the commands in the scene.
    pub fn iter( &self ) -> core::slice::Iter< '_, RenderCommand >
    {
      self.commands.iter()
    }

    /// Returns true if the scene contains no commands.
    #[ must_use ]
    pub fn is_empty( &self ) -> bool
    {
      self.commands.is_empty()
    }

    /// Clears all commands from the scene.
    pub fn clear( &mut self )
    {
      self.commands.clear();
    }

    /// Gets the scene identifier if set.
    #[ must_use ]
    pub fn id( &self ) -> Option< &str >
    {
      self.id.as_deref()
    }

    /// Sets the scene identifier.
    pub fn set_id( &mut self, id: impl Into< String > )
    {
      self.id = Some( id.into() );
    }

    /// Returns an iterator over the commands in the scene.
    pub fn commands( &self ) -> SceneCommandIter< '_ >
    {
      SceneCommandIter
      {
        commands: self.commands.iter(),
      }
    }

    /// Returns a mutable iterator over the commands in the scene.
    pub fn commands_mut( &mut self ) -> SceneCommandIterMut< '_ >
    {
      SceneCommandIterMut
      {
        commands: self.commands.iter_mut(),
      }
    }

    /// Gets a command by index.
    #[ must_use ]
    pub fn get( &self, index: usize ) -> Option< &RenderCommand >
    {
      self.commands.get( index )
    }

    /// Gets a mutable reference to a command by index.
    #[ must_use ]
    pub fn get_mut( &mut self, index: usize ) -> Option< &mut RenderCommand >
    {
      self.commands.get_mut( index )
    }

    /// Removes a command at the specified index.
    pub fn remove( &mut self, index: usize ) -> Option< RenderCommand >
    {
      if index < self.commands.len()
      {
        Some( self.commands.remove( index ) )
      }
      else
      {
        None
      }
    }

    /// Inserts a command at the specified index.
    pub fn insert( &mut self, index: usize, command: RenderCommand )
    {
      if index <= self.commands.len()
      {
        self.commands.insert( index, command );
      }
    }

    /// Queries commands by type (FR-A6).
    /// Returns all Line commands in the scene.
    #[ must_use ]
    pub fn query_lines( &self ) -> QueryResult< '_ >
    {
      let commands = self.commands.iter()
        .filter( |cmd| matches!( cmd, RenderCommand::Line( _ ) ) )
        .collect();
      QueryResult { commands }
    }

    /// Queries commands by type (FR-A6).
    /// Returns all Curve commands in the scene.
    #[ must_use ]
    pub fn query_curves( &self ) -> QueryResult< '_ >
    {
      let commands = self.commands.iter()
        .filter( |cmd| matches!( cmd, RenderCommand::Curve( _ ) ) )
        .collect();
      QueryResult { commands }
    }

    /// Queries commands by type (FR-A6).
    /// Returns all Text commands in the scene.
    #[ must_use ]
    pub fn query_text( &self ) -> QueryResult< '_ >
    {
      let commands = self.commands.iter()
        .filter( |cmd| matches!( cmd, RenderCommand::Text( _ ) ) )
        .collect();
      QueryResult { commands }
    }

    /// Queries commands by type (FR-A6).
    /// Returns all Tilemap commands in the scene.
    #[ must_use ]
    pub fn query_tilemaps( &self ) -> QueryResult< '_ >
    {
      let commands = self.commands.iter()
        .filter( |cmd| matches!( cmd, RenderCommand::Tilemap( _ ) ) )
        .collect();
      QueryResult { commands }
    }

    /// Queries commands by type (FR-A6).
    /// Returns all `ParticleEmitter` commands in the scene.
    #[ must_use ]
    pub fn query_particle_emitters( &self ) -> QueryResult< '_ >
    {
      let commands = self.commands.iter()
        .filter( |cmd| matches!( cmd, RenderCommand::ParticleEmitter( _ ) ) )
        .collect();
      QueryResult { commands }
    }

    /// General query method that accepts a predicate function.
    #[ must_use ]
    pub fn query_where< F >( &self, predicate: F ) -> QueryResult< '_ >
    where
      F: Fn( &RenderCommand ) -> bool,
    {
      let commands = self.commands.iter()
        .filter( |cmd| predicate( cmd ) )
        .collect();
      QueryResult { commands }
    }

    /// Returns statistics about command types in the scene.
    #[ must_use ]
    pub fn stats( &self ) -> SceneStats
    {
      let mut stats = SceneStats::default();

      for command in &self.commands
      {
        match command
        {
          RenderCommand::Line( _ ) => stats.line_count += 1,
          RenderCommand::Curve( _ ) => stats.curve_count += 1,
          RenderCommand::Text( _ ) => stats.text_count += 1,
          RenderCommand::Tilemap( _ ) => stats.tilemap_count += 1,
          RenderCommand::ParticleEmitter( _ ) => stats.particle_emitter_count += 1,
          RenderCommand::Geometry2DCommand( _ ) => stats.geometry2d_count += 1,
          RenderCommand::SpriteCommand( _ ) => stats.sprite_count += 1,
        }
      }

      stats.total_count = self.commands.len();
      stats
    }
  }

  impl Default for Scene
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl< 'a > Iterator for SceneCommandIter< 'a >
  {
    type Item = &'a RenderCommand;

    fn next( &mut self ) -> Option< Self::Item >
    {
      self.commands.next()
    }

    fn size_hint( &self ) -> ( usize, Option< usize > )
    {
      self.commands.size_hint()
    }
  }

  impl< 'a > Iterator for SceneCommandIterMut< 'a >
  {
    type Item = &'a mut RenderCommand;

    fn next( &mut self ) -> Option< Self::Item >
    {
      self.commands.next()
    }

    fn size_hint( &self ) -> ( usize, Option< usize > )
    {
      self.commands.size_hint()
    }
  }

  impl< 'a > QueryResult< 'a >
  {
    /// Returns the number of commands in the query result.
    #[ must_use ]
    pub fn len( &self ) -> usize
    {
      self.commands.len()
    }

    /// Returns true if the query result is empty.
    #[ must_use ]
    pub fn is_empty( &self ) -> bool
    {
      self.commands.is_empty()
    }

    /// Returns an iterator over the commands in the query result.
    pub fn iter( &self ) -> core::slice::Iter< '_, &'a RenderCommand >
    {
      self.commands.iter()
    }

    /// Gets a command by index from the query result.
    #[ must_use ]
    pub fn get( &self, index: usize ) -> Option< &'a RenderCommand >
    {
      self.commands.get( index ).copied()
    }
  }

  /// Statistics about command types in a scene.
  #[ derive( Debug, Clone, Default, PartialEq ) ]
  pub struct SceneStats
  {
    /// Total number of commands in the scene.
    pub total_count : usize,
    /// Number of Line commands.
    pub line_count : usize,
    /// Number of Curve commands.
    pub curve_count : usize,
    /// Number of Text commands.
    pub text_count : usize,
    /// Number of Tilemap commands.
    pub tilemap_count : usize,
    /// Number of `ParticleEmitter` commands.
    pub particle_emitter_count : usize,
    /// Number of `Geometry2D` commands.
    pub geometry2d_count : usize,
    /// Number of `Sprite` commands.
    pub sprite_count : usize,
  }

}

#[ cfg( feature = "enabled" ) ]
pub use private::*;
