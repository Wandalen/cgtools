# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 13.10.2025

Changes:

- Removed Color, TweenBuilder and AnimationBuilder

Deprecation:

- Color, TweenBuilder and AnimationBuilder are deprecated

Migration:

If you have used Color up to this point then implement your own analog. If you have used TweenBuilder or AnimationBuilder up to this point then you need now use Animation, Sequencer, Sequence, Tween for animation creation. New framework makes wider opportunities to create and use animations.

## 24.10.2025

Changes:

- Moved traits to new module
- Added methods current_id_get, tweens_get to Sequencer
- Added Clone trait for Sequencer, Sequence and all EasingFunctions impls
- Added methods get_mut, get_value to Tween
