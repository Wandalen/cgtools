# behaviour_tree

Behavior tree implementation for hierarchical AI decision-making in games.

## Overview

A behavior tree is composed of nodes representing actions, conditions, and
control-flow logic. The tree is executed from the root; each node returns a
status indicating success, failure, or that it's still running (for
multi-frame actions).

## Node Types

- **Action nodes** — perform specific actions (move, attack, patrol, wait)
- **Condition nodes** — check game state via blackboard value comparisons
- **Composite nodes** — control execution flow: `Sequence`, `Selector`, `Parallel`
- **Decorator nodes** — modify child behavior: `Repeat`, `Invert`, `Cooldown`

## Usage

```rust
use behaviour_tree::*;

// Create a simple patrol behavior using blackboard values
let mut patrol_tree = BehaviorTreeBuilder::new()
.sequence
(
  vec!
  [
    blackboard_set( "target_x", 10 ),
    blackboard_set( "target_y", 10 ),
    wait( 2.0 ), // Wait 2 seconds
    blackboard_set( "target_x", 5 ),
    blackboard_set( "target_y", 5 ),
    wait( 2.0 ),
  ]
)
.build();

// Execute the behavior tree
let mut context = BehaviorContext::new();
let status = patrol_tree.execute( &mut context );
```

## Testing

```bash
cargo test -p behaviour_tree
```

## Directory Layout

| Path | Responsibility |
|------|----------------|
| `src/` | Crate source — node types, builder, blackboard context |
| `tests/` | Integration tests |
| `readme.md` | This file — user-facing entry point |
