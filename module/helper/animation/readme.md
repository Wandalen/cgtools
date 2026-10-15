# 🏃 animation

> **Library for values interpolation and complex values sequence smooth transition**

A general purpose library for working with animatable values. Functionality includes: tween system for value interpolation for set of types, extendable set of easing functions for a wide variety of value behaviors, sequencer system for coordination of many values interpolation in every time moment.

## ✨ Features

### ➡️ **Interpolation**
- **Tween** - system for one value interpolation playback
- **Animatable** - trait for values that can be interpolated

### 📈 **Easing**
- **EasingFunction** - Interface for creating new easing functions
- **Linear** - Linear function
- **Step** - Step function
- **Cubic** - Cubic spline function

### 🎞️ **Sequencer**
- **Sequencer** - Core system for animations playback
- **AnimatablePlayer** - trait for types that can be used in [`Sequencer`]

## 📦 Installation

Add to your `Cargo.toml`:
```toml
animation = { workspace = true }
```

## 🚀 Quick Start

### Tween usage

```rust
use animation::interpolation::Tween;
use animation::easing::{ EasingBuilder, Linear };
use animation::AnimatablePlayer;

let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() )
.with_repeat( 1 ).with_yoyo( true );

// First loop: 0.0 -> 10.0
let val1 = tween.update( 0.5 ); // 5.0
tween.update( 0.5 );
tween.value_get(); // 10.0
tween.current_repeat(); // 1

// Second loop: 10.0 -> 0.0 (yoyo)
let val2 = tween.update( 0.5 ); // 5.0
tween.update( 0.5 );
tween.value_get(); // 0.0
tween.is_completed(); // true
```

### Sequencer usage

```rust
use animation::interpolation::Tween;
use animation::sequencer::Sequencer;
use animation::easing::{ Linear, EasingBuilder };
use animation::AnimatablePlayer;

let mut sequencer = Sequencer::new();
sequencer.insert
(
  "test",
  Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::build() )
);

sequencer.update( 0.5 );
sequencer.time(); // 0.5
sequencer.get::< Tween< f32 > >( "test" ).unwrap().value_get(); // 5.0

sequencer.reset();

sequencer.time(); // 0.0
sequencer.state(); // AnimationState::Running
sequencer.get::< Tween< f32 > >( "test" ).unwrap().value_get(); // 0.0

sequencer.update( 1.0 );
sequencer.is_completed(); // true
```

### Easing functions usage

```rust
use animation::easing::
{
  EasingFunction,
  EasingBuilder,
  Linear,
  Step,
  cubic::EaseInSine
};

let f = Linear::build();
let value = f.apply( 0.0, 1.0, 0.5 );

// You can choose steps count that split range [`0.0..1.0`]
// on steps ranges that have const values
let f = Step::new( 5.0 );
let value = f.apply( 0.0, 1.0, 0.5 );

// Returns [`Cubic`] instance with specific behavior
let f = EaseInSine::build();
let value = f.apply( 0.0, 1.0, 0.5 );
```

## 📖 API Reference

### Core Components

| Component | Purpose | Key Methods |
|-----------|---------|-------------|
| `Sequencer` | Named, heterogeneous player collection running in parallel | `new()`, `insert()`, `get()`, `update()`, `progress()` |
| `Sequence` | Ordered, homogeneous player chain running one at a time | `new()`, `current_get()`, `update()`, `progress()` |
| `Tween` | One value interpolation player | `new()`, `update()`, `value_get()`, `progress()` |
| `EasingFunction` | Easing function trait | `apply()` |

## 🎯 Use Cases

- **Lottie animations** - Own lottie animations playback
- **Skeletal animations** - Control every node transforms in skeleton in every time moment
- **Simple animations** - Create animated values for color, object movement without sequencer
