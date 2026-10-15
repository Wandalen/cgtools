# Stealth Game

**Keywords:** Field of View, Stealth, ECS, Line of Sight, tiles_tools

A small stealth game built on `tiles_tools`' field-of-view system: guards patrol and detect the player via line-of-sight, the player has stealth mechanics and hiding spots, and torches/lamps contribute dynamic lighting that affects visibility. Uses the crate's `ecs-systems` feature and references `hecs::Entity` directly for guard/player entity handles.

*(No showcase — console/logic demo, no visual output)*

**[How to run](../../how_to_run.md)**

**References:**

* [hecs — ECS library]
* [tiles_tools on crates.io]

[hecs — ECS library]: https://docs.rs/hecs/
[tiles_tools on crates.io]: https://crates.io/crates/tiles_tools
