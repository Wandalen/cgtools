//! Typed lookup catalog for object and state handles.
//!
//! Building a catalog up-front trades the per-call-site `.expect()` /
//! `.unwrap()` tax on [`crate::scene::Scene::object`] /
//! [`crate::scene::Scene::state`] for a single fail-fast validation
//! during adapter init. Every required id is checked against the spec;
//! all misses are reported together so callers see the full picture
//! rather than one missing id at a time.
//!
//! The catalog stores `ObjectHandle` / `StateHandle` by id; the
//! `object` / `state` accessors are panic-on-miss because the missing
//! cases have already been ruled out by `build()`. Ad-hoc lookups
//! through `Scene::object` / `Scene::state` remain `Option<…>`-typed
//! for callers that genuinely don't know whether an id exists.

mod private
{
  use rustc_hash::FxHashMap as HashMap;
  use crate::instance::{ ObjectHandle, StateHandle };
  use crate::scene::Scene;

  /// Compile-time-validated lookup table from string ids to scene handles.
  ///
  /// Construct via [`Scene::catalog`] → [`CatalogBuilder`] →
  /// [`CatalogBuilder::build`]. Hot-path consumers cache handles in a
  /// `Catalog` during init; the reconcile loop then resolves ids via
  /// `.object(id)` / `.state(obj, state)` without `Option` ceremony.
  #[ derive( Debug, Clone ) ]
  pub struct Catalog
  {
    objects : HashMap< String, ObjectHandle >,
    states : HashMap< ( String, String ), StateHandle >,
  }

  impl Catalog
  {
    /// Resolve an object id that was required at build time.
    ///
    /// # Panics
    ///
    /// Panics if `id` was not declared via
    /// [`CatalogBuilder::require_object`] (or implicitly via
    /// [`CatalogBuilder::require_state`]) before `build()` was called.
    /// Forgetting to declare an id is a programmer error — the missing
    /// case is the whole reason `Catalog` exists.
    #[ inline ]
    #[ must_use ]
    pub fn object( &self, id : &str ) -> ObjectHandle
    {
      *self.objects.get( id ).unwrap_or_else
      (
        || panic!( "Catalog::object: id {id:?} was not required at build time" )
      )
    }

    /// Resolve a `(object, state)` pair that was required at build time.
    ///
    /// # Panics
    ///
    /// Panics if `(obj, state)` was not declared via
    /// [`CatalogBuilder::require_state`] before `build()` was called.
    #[ inline ]
    #[ must_use ]
    pub fn state( &self, obj : &str, state : &str ) -> StateHandle
    {
      let key = ( obj.to_owned(), state.to_owned() );
      *self.states.get( &key ).unwrap_or_else
      (
        || panic!( "Catalog::state: ({obj:?}, {state:?}) was not required at build time" )
      )
    }

    /// Try to resolve an object id without panicking. Returns `None` if
    /// the id was not required at build time.
    #[ inline ]
    #[ must_use ]
    pub fn try_object( &self, id : &str ) -> Option< ObjectHandle >
    {
      self.objects.get( id ).copied()
    }

    /// Try to resolve a `(object, state)` pair without panicking.
    #[ inline ]
    #[ must_use ]
    pub fn try_state( &self, obj : &str, state : &str ) -> Option< StateHandle >
    {
      let key = ( obj.to_owned(), state.to_owned() );
      self.states.get( &key ).copied()
    }
  }

  /// Collected misses returned by [`CatalogBuilder::build`].
  ///
  /// Distinct from [`crate::error::ValidationError`] because catalog
  /// validation is a *consumer* concern — the spec on its own is
  /// internally consistent; the adapter just needs the subset of ids
  /// it intends to touch.
  #[ derive( Debug, Clone ) ]
  pub struct CatalogError
  {
    /// Every requested object id that the spec does not declare.
    pub missing_objects : Vec< String >,
    /// Every requested `(object, state)` pair where either the object
    /// or the state is missing. Stored as a pair so the error message
    /// makes the relationship obvious.
    pub missing_states : Vec< ( String, String ) >,
  }

  impl core::fmt::Display for CatalogError
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      writeln!
      (
        f,
        "Catalog build failed: {} missing object(s), {} missing state(s)",
        self.missing_objects.len(),
        self.missing_states.len(),
      )?;
      for id in &self.missing_objects
      {
        writeln!( f, "  - object: {id:?}" )?;
      }
      for ( obj, state ) in &self.missing_states
      {
        writeln!( f, "  - state:  ({obj:?}, {state:?})" )?;
      }
      Ok( () )
    }
  }

  impl core::error::Error for CatalogError {}

  /// Fluent builder for [`Catalog`]. Construct via [`Scene::catalog`].
  ///
  /// Required ids are accumulated by `require_*` calls; [`Self::build`]
  /// resolves every requirement against the scene's spec and returns
  /// either the populated `Catalog` or a [`CatalogError`] listing
  /// every miss together.
  pub struct CatalogBuilder< 'a >
  {
    scene : &'a Scene,
    objects : Vec< String >,
    states : Vec< ( String, String ) >,
  }

  impl< 'a > CatalogBuilder< 'a >
  {
    /// Internal constructor — call [`Scene::catalog`] instead.
    #[ doc( hidden ) ]
    #[ must_use ]
    pub fn new( scene : &'a Scene ) -> Self
    {
      Self { scene, objects : Vec::new(), states : Vec::new() }
    }

    /// Declare that the catalog must resolve the object id `id`.
    #[ must_use ]
    pub fn require_object( mut self, id : impl Into< String > ) -> Self
    {
      self.objects.push( id.into() );
      self
    }

    /// Declare that the catalog must resolve `state` on object `obj`.
    /// Implicitly also requires `obj` itself; callers do not need a
    /// separate `require_object` for the same id.
    #[ must_use ]
    pub fn require_state
    (
      mut self,
      obj : impl Into< String >,
      state : impl Into< String >,
    ) -> Self
    {
      let obj = obj.into();
      let state = state.into();
      self.objects.push( obj.clone() );
      self.states.push( ( obj, state ) );
      self
    }

    /// Resolve every required id against the scene's spec.
    ///
    /// On success returns a [`Catalog`] whose `.object()` / `.state()`
    /// lookups for any required id are infallible. On failure returns
    /// a [`CatalogError`] enumerating *every* missing id at once.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] when any required object id is not
    /// declared in the spec, or any required state name is not present
    /// on its object. The error carries the complete list — partial
    /// repair is intentional so adapters can patch all the typos in
    /// one pass.
    pub fn build( self ) -> Result< Catalog, CatalogError >
    {
      let mut objects : HashMap< String, ObjectHandle > = HashMap::default();
      let mut missing_objects : Vec< String > = Vec::new();
      let mut seen_missing_objects : rustc_hash::FxHashSet< String > = rustc_hash::FxHashSet::default();

      for id in &self.objects
      {
        if objects.contains_key( id ) || seen_missing_objects.contains( id )
        {
          continue;
        }
        if let Some( h ) = self.scene.object( id )
        {
          objects.insert( id.clone(), h );
        }
        else
        {
          seen_missing_objects.insert( id.clone() );
          missing_objects.push( id.clone() );
        }
      }

      let mut states : HashMap< ( String, String ), StateHandle > = HashMap::default();
      let mut missing_states : Vec< ( String, String ) > = Vec::new();

      for ( obj, state ) in &self.states
      {
        let Some( &obj_handle ) = objects.get( obj )
        else
        {
          // The object itself was missing — already reported. Don't
          // double-report the state miss.
          continue;
        };
        if let Some( h ) = self.scene.state( obj_handle, state )
        {
          states.insert( ( obj.clone(), state.clone() ), h );
        }
        else
        {
          missing_states.push( ( obj.clone(), state.clone() ) );
        }
      }

      if missing_objects.is_empty() && missing_states.is_empty()
      {
        Ok( Catalog { objects, states } )
      }
      else
      {
        Err( CatalogError { missing_objects, missing_states } )
      }
    }
  }
}

mod_interface::mod_interface!
{
  exposed use Catalog;
  exposed use CatalogBuilder;
  exposed use CatalogError;
}
