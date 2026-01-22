# ECS Library Decision Document

**Version:** 1.0  
**Date:** 2025-08-08  
**Author:** Claude Code  

## Executive Summary

After evaluating the primary ECS libraries available in the Rust ecosystem, **`hecs`** has been selected as the core ECS dependency for the tiles_tools crate. This decision prioritizes compilation speed, simplicity, and alignment with the project's architectural principles while avoiding the complexity and dependency overhead of bevy_ecs.

## Evaluation Criteria

The following criteria were used to evaluate each ECS library:

| Criterion | Weight | Description |
|-----------|--------|-------------|
| **Performance** | High | Runtime performance for typical game operations |
| **API Ergonomics** | High | Developer experience and ease of use |
| **Compile Times** | High | Impact on build times |
| **Feature Set** | Medium | Advanced features like change detection |
| **Maintenance** | Medium | Community support and active development |
| **Dependency Overhead** | High | Size of dependency tree |
| **Workspace Compatibility** | High | Alignment with existing cgtools ecosystem |

## Library Analysis

### HECS

| Aspect | Rating | Analysis |
|--------|--------|----------|
| **Performance** | Excellent | Archetype-based storage with excellent cache locality |
| **API Ergonomics** | Good | Clean, minimal API that's easy to understand |
| **Compile Times** | Excellent | Minimal dependency tree, fast compilation |
| **Feature Set** | Good | Core ECS functionality without bloat |
| **Maintenance** | Good | Stable, well-maintained by the Rust gamedev community |
| **Dependency Overhead** | Excellent | Very lightweight dependency tree |
| **Workspace Compatibility** | Excellent | Aligns with cgtools' preference for minimal dependencies |

**Strengths:**
- Lightweight with minimal dependencies
- Fast compile times critical for development velocity
- Simple, clean API that follows Rust idioms
- Excellent performance characteristics
- No unnecessary features that would complicate our abstraction layer

**Weaknesses:**
- Fewer advanced features compared to bevy_ecs
- Less extensive documentation ecosystem

### Bevy ECS

| Aspect | Rating | Analysis |
|--------|--------|----------|
| **Performance** | Excellent | High-performance with advanced scheduling |
| **API Ergonomics** | Excellent | Very ergonomic with powerful query system |
| **Compile Times** | Poor | Heavy dependency tree impacts compilation |
| **Feature Set** | Excellent | Rich feature set including change detection |
| **Maintenance** | Excellent | Very active development and community |
| **Dependency Overhead** | Poor | Large dependency tree |
| **Workspace Compatibility** | Poor | **Conflicts with user requirement to avoid bevy** |

**Strengths:**
- Most feature-complete ECS in Rust
- Excellent performance and scheduling
- Rich ecosystem and documentation

**Weaknesses:**
- **Explicitly excluded by user requirements**
- Heavy dependency tree that would slow compilation
- Overkill for our needs - many features we wouldn't use
- Would conflict with cgtools' minimal dependency philosophy

### Specs

| Aspect | Rating | Analysis |
|--------|--------|----------|
| **Performance** | Good | Mature, well-optimized |
| **API Ergonomics** | Fair | Older API design, more verbose |
| **Compile Times** | Good | Moderate dependency overhead |
| **Feature Set** | Good | Mature feature set |
| **Maintenance** | Poor | In maintenance mode, minimal active development |
| **Dependency Overhead** | Fair | Moderate dependency tree |
| **Workspace Compatibility** | Fair | Acceptable but not optimal |

**Strengths:**
- Proven track record in production games
- Stable and mature

**Weaknesses:**
- In maintenance mode with limited future development
- More verbose API compared to modern alternatives
- Architecture feels dated compared to newer ECS libraries

## Decision Matrix

| Library | Performance | API Ergonomics | Compile Times | Maintenance | Dep. Overhead | **Total Score** |
|---------|-------------|----------------|---------------|-------------|---------------|-----------------|
| **HECS** | 5 | 4 | 5 | 4 | 5 | **23/25** |
| Bevy ECS | 5 | 5 | 1 | 5 | 1 | **17/25** |
| Specs | 4 | 3 | 4 | 2 | 3 | **16/25** |

## Recommendation: HECS

**HECS** is recommended as the core ECS library for the following reasons:

1. **User Requirements Compliance**: Avoids bevy_ecs as explicitly requested
2. **Compilation Performance**: Critical for development velocity in the cgtools ecosystem
3. **Simplicity**: Easier to abstract and maintain our own API layer over it
4. **Dependency Discipline**: Aligns with cgtools' philosophy of minimal dependencies
5. **Performance**: Still provides excellent runtime performance without the overhead

## Architecture Implications

Using HECS will require:

1. **Abstraction Layer**: Create a thin wrapper in `core::ecs` module
2. **Entity Management**: Implement our own Entity newtype for type safety
3. **Component Management**: Leverage HECS's component storage directly
4. **Query Interface**: Build our query API on top of HECS's query system

## Implementation Plan

The chosen library will be integrated according to the Design Rulebook's **Dependency Inversion Principle**:

```rust
// core::ecs module structure
pub struct Entity(hecs::Entity);
pub struct World(hecs::World);

impl World {
    pub fn entity_create(&mut self) -> Entity { /* ... */ }
    pub fn component_add<C>(&mut self, entity: Entity, component: C) { /* ... */ }
    // ... other wrapper methods
}
```

This approach ensures:
- Our codebase depends on abstractions, not concrete implementations
- Future ECS library changes require minimal refactoring
- Type safety through newtype wrappers
- Clean separation between our API and the underlying ECS

## Conclusion

HECS provides the optimal balance of performance, simplicity, and maintainability for the tiles_tools project while respecting user requirements and architectural constraints. This decision supports the project's goals of fast compilation, minimal dependencies, and clean abstractions.