mod private
{
  use std::{ rc::Rc, cell::RefCell };
  use std::collections::HashMap;
  use animation::{ Tween, AnimatablePlayer, Sequencer };
  use crate::webgl::
  {
    Node,
    animation::
    {
      Pose,
      Transition,
      AnimatableComposition
    }
  };

  /// Controls transition process from one [`AnimationNode`] to another
  struct AnimationEdge
  {
    /// Edge name
    name : Box< str >,
    /// Next [`AnimationNode`] after transition
    next : Rc< RefCell< AnimationNode > >,
    /// Transition behavior
    transition : Transition,
    /// Condition closure that manages when apply transition. This implementation
    /// assumes that transition may happen when [`Node`] or [`CharacterControls`]
    /// change theirs state that can be identified by past and present [`Node`]'s [`Pose`].
    condition : fn( &Pose, &Pose ) -> bool
  }

  impl AnimationEdge
  {
    /// Create new [`AnimationEdge`]
    fn new
    (
      name : Box< str >,
      next : &Rc< RefCell< AnimationNode > >,
      transition : Transition,
      condition : fn( &Pose, &Pose ) -> bool
    )
    -> Self
    {
      Self
      {
        name,
        next : next.clone(),
        transition : transition.clone(),
        condition
      }
    }

    /// Returns next [`AnimationNode`]
    fn next_get( &self ) -> Rc< RefCell< AnimationNode > >
    {
      self.next.clone()
    }

    /// Check if [`Self::condition`] returns true
    fn is_triggered( &self, past : &Pose, current : &Pose ) -> bool
    {
      ( self.condition )( past, current )
    }

    /// Get [`Self::transition`] as reference
    fn transition_as_ref( &self ) -> &Transition
    {
      &self.transition
    }

    /// Get [`Self::transition`] as mutable reference
    fn transition_as_mut( &mut self ) -> &mut Transition
    {
      &mut self.transition
    }
  }

  impl Clone for AnimationEdge
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        name : self.name.clone(),
        next : self.next.clone(),
        transition : self.transition.clone(),
        condition : self.condition
      }
    }
  }

  /// Controls animation state of [`AnimationGraph`] at certain stage
  #[ derive( Clone ) ]
  struct AnimationNode
  {
    /// Node name
    name : Box< str >,
    /// Animation played when this [`AnimationNode`] is current
    /// and transition is not performed yet
    animation : Sequencer,
    /// [`AnimationEdge`] that controls animation state now. [`AnimationNode`]
    /// controls also transition process to next [`AnimationNode`].
    in_process : Option< Rc< RefCell< AnimationEdge > > >,
    /// List of [`AnimationEdge`]'s for transition from one [`AnimationNode`] to another
    edges : HashMap< Box< str >, Rc< RefCell< AnimationEdge > > >
  }

  /// Directed graph that controls animation state of some [`crate::webgl::Skeleton`]
  #[ derive( Clone ) ]
  pub struct AnimationGraph
  {
    /// [`AnimationNode`] that are currently played on
    /// cycle for related [`crate::webgl::Skeleton`]
    current : Option< Rc< RefCell< AnimationNode > > >,
    /// [`Node`]'s animated by this [`AnimationGraph`]
    nodes : HashMap< Box< str >, Rc< RefCell< Node > > >,
    /// List of [`AnimationNode`] that is part of animation
    /// state update process
    animation_nodes : HashMap< Box< str >, Rc< RefCell< AnimationNode > > >,
    /// Last [`Pose`] of related [`crate::webgl::Skeleton`]
    last_pose : Option< Pose >
  }

  impl AnimationGraph
  {
    /// Creates new [`AnimationGraph`]
    pub fn new( nodes : &HashMap< Box< str >, Rc< RefCell< Node > > > ) -> Self
    {
      Self
      {
        current : None,
        nodes : nodes.clone(),
        animation_nodes : HashMap::new(),
        last_pose : None
      }
    }

    /// Gets current [`AnimationNode`] name
    pub fn current_name_get( &self ) -> Option< Box< str > >
    {
      self.current.as_ref().map( | n | n.borrow().name.clone() )
    }

    /// Sets current [`AnimationNode`]
    pub fn current_set( &mut self, name : Box< str > )
    {
      self.current = self.animation_nodes.get( &name ).map( | n | n.clone() );
    }

    /// Add new [`AnimationNode`]
    pub fn node_add( &mut self, name : Box< str >, animation : Sequencer )
    {
      let node = AnimationNode
      {
        name : name.clone(),
        animation,
        in_process : None,
        edges : HashMap::new(),
      };
      self.animation_nodes.insert( name, Rc::new( RefCell::new( node ) ) );
    }

    /// Remove [`AnimationNode`]
    pub fn node_remove( &mut self, name : Box< str > )
    {
      self.animation_nodes.remove( &name );
    }

    /// Add new [`AnimationEdge`]
    pub fn edge_add
    (
      &self,
      a : Box< str >,
      b : Box< str >,
      name : Box< str >,
      tween : Tween< f64 >,
      condition : fn( &Pose, &Pose ) -> bool
    )
    {
      let Some( a ) = self.animation_nodes.get( &a )
      else
      {
        return;
      };

      let Some( b ) = self.animation_nodes.get( &b )
      else
      {
        return;
      };

      let transition = Transition::new
      (
        a.borrow().animation.clone(),
        b.borrow().animation.clone(),
        tween
      );

      let edge = AnimationEdge::new
      (
        name.clone(),
        b,
        transition,
        condition
      );

      a.borrow_mut().edges.insert( name, Rc::new( RefCell::new( edge ) ) );
    }

    /// Remove [`AnimationEdge`]
    pub fn edge_remove( &self, node_name : Box< str >, name : Box< str > )
    {
      let Some( node ) = self.animation_nodes.get( &node_name )
      else
      {
        return;
      };

      node.borrow_mut().edges.remove( &name );
    }

    /// Gets map of animated [`Node`]'s
    pub fn animated_nodes_get( &self ) -> &HashMap< Box< str >, Rc< RefCell< Node > > >
    {
      &self.nodes
    }
  }

  impl AnimatableComposition for AnimationGraph
  {
    fn update( &mut self, delta_time : f64 )
    {
      let node_list = self.nodes.values().cloned().collect::< Vec< _ > >();

      let current_pose = Pose::new(node_list.as_slice() );

      if let Some( current ) = &self.current
      {
        let mut triggered_edge = None;
        for ( _, edge ) in &current.borrow().edges
        {
          if edge.borrow().is_triggered( self.last_pose.as_ref().unwrap_or( &current_pose ), &current_pose )
          {
            triggered_edge = Some( edge.clone() );
            break;
          }
        }

        if let Some( edge ) = triggered_edge.as_ref()
        {
          let time = current.borrow().animation.time();
          edge.borrow_mut().transition.start_as_mut().update( time );
        }

        current.borrow_mut().animation.reset();
        current.borrow_mut().in_process = triggered_edge;
      }

      self.last_pose = Some( current_pose );

      let mut is_transited = false;
      if let Some( current ) = &self.current
      {
        if let Some( edge ) = &current.borrow().in_process
        {
          if edge.borrow().transition_as_ref().tween_get().is_completed()
          {
            is_transited = true;
          }
          else
          {
            edge.borrow_mut().transition_as_mut().update( delta_time );
          }
        }
        else
        {
          if current.borrow().animation.is_completed()
          {
            current.borrow_mut().animation.reset();
          }

          current.borrow_mut().animation.update( delta_time );
        }
      }

      if is_transited
      {
        let old = self.current.as_ref().unwrap().clone();
        let next = self.current.as_ref().unwrap().borrow().in_process.as_ref().unwrap().borrow().next_get();
        let time = old.borrow().in_process.as_ref().unwrap().borrow().transition_as_ref().end_as_ref().time();
        next.borrow_mut().animation.update( time );
        self.current = Some( next );
        old.borrow().in_process.as_ref().unwrap().borrow_mut().transition_as_mut().reset();
        old.borrow_mut().in_process = None;
      }
    }

    fn as_any( &self ) -> &dyn core::any::Any
    {
      self
    }

    fn as_any_mut( &mut self ) -> &mut dyn core::any::Any
    {
      self
    }

    fn set( &self, nodes : &HashMap< Box< str >, Rc< RefCell< Node > > > )
    {
      if let Some( current ) = &self.current
      {
        if let Some( edge ) = &current.borrow().in_process
        {
          edge.borrow().transition_as_ref().set( nodes );
        }
        else
        {
          current.borrow().animation.set( nodes );
        }
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    AnimationGraph
  };
}
