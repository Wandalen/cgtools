# spec

- **Name:** Agnostic 2D Render Engine
- **Version:** 1.0 (Final)
- **Date:** 2025-08-08

### **Table of Contents**

**Part I: Public Contract (The Rendering API)**
1.  [Project Goal](#1-project-goal)
2.  [Problem Solved](#2-problem-solved)
3.  [Ubiquitous Language (Vocabulary)](#3-ubiquitous-language-vocabulary)
4.  [Deliverables](#4-deliverables)
5.  [Success Metrics](#5-success-metrics)
6.  [Vision & Scope](#6-vision--scope)
7.  [System Actors](#7-system-actors)
8.  [Functional Requirements: Scene Definition](#8-functional-requirements-scene-definition)
9.  [Functional Requirements: CLI](#9-functional-requirements-cli)
10. [Non-Functional Requirements](#10-non-functional-requirements)

**Part II: Internal Design (Backend Implementation)**
11. [System Architecture (Ports & Adapters)](#11-system-architecture-ports--adapters)
12. [External Dependencies Analysis](#12-external-dependencies-analysis)
13. [Core Rendering Ports (Traits)](#13-core-rendering-ports-traits)
14. [Required Backend Adapters](#14-required-backend-adapters)
15. [Architectural & Flow Diagrams](#15-architectural--flow-diagrams)

**Part III: Project & Process Governance**
16. [Core Principles of Development](#16-core-principles-of-development)
17. [Open Questions](#17-open-questions)

**Appendix: Addendum**
- [Developer Implementation Notes](#appendix-addendum)

---

**Part I: Public Contract (The Rendering API)**

### 1. Project Goal
To create a high-performance, backend-agnostic 2D rendering engine for Rust, designed for maximum flexibility. The engine will decouple scene definition from the rendering implementation, allowing a single scene to be rendered across multiple backends.

### 2. Problem Solved
Developers are often locked into a single rendering backend, making it difficult to reuse rendering logic across different platforms or for different purposes. This project solves that by providing a clean abstraction layer, allowing a developer to define a scene once and render it anywhere.

### 3. Ubiquitous Language (Vocabulary)
| Term | Definition |
| :--- | :--- |
| **Port** | An interface (a Rust `trait`) that defines a set of rendering capabilities. |
| **Adapter** | A concrete implementation of one or more `Port` traits for a specific `Backend`. |
| **Backend** | The underlying graphics library or technology used by an `Adapter`. |
| **Scene** | A description of everything to be rendered, composed of a `Command Queue`. |
| **Render Command** | A lightweight, POD `struct` that describes a single primitive to be drawn. |

### 4. Deliverables
1.  **Published Core Crate (`agnostic_renderer`):** The primary library with the public API and `Port` traits.
2.  **Published Adapter Crates:** Separate crates for each backend (`renderer_adapter_wgpu`, `_webgl`, `_svg`, `_terminal`).
3.  **Published CLI Crate (`are_cli`):** The interactive command-line tool.
4.  **Source Code Repository:** A Git monorepo containing all crates.
5.  **Comprehensive API Documentation:** Publicly hosted `cargo doc` documentation.
6.  **Usage Examples & Gallery:** Examples showing the same scene rendered with different backends.

### 5. Success Metrics
| Category | Metric | Target |
| :--- | :--- | :--- |
| **Adoption** | Downloads on `crates.io` | Achieve 500 downloads within 6 months of the 1.0 release. |
| **Extensibility** | Community Adapters | At least one community-contributed backend adapter within the first year. |
| **Usability** | Time to Proficiency | A new developer can create a basic 'hello world' adapter in under 4 hours. |

### 6. Vision & Scope
#### 6.1. Vision
To provide the Rust ecosystem with a uniquely flexible and decoupled 2D rendering engine, enabling developers to "write once, render anywhere."

#### 6.2. In Scope (for Version 1.0)
-   A command-based scene definition API.
-   Core primitives: `Line`, `Curve`, `Text`, `Tilemap`, `ParticleEmitter`.
-   A "Ports and Adapters" architecture.
-   Backend adapters for `wgpu`, `webgl`, `svg`, and `terminal`.
-   A `unilang`-based CLI for interactive editing and headless rendering.

#### 6.3. Out of Scope (for Version 1.0)
-   3D Rendering.
-   A full game engine (physics, audio, etc.).
-   Advanced rendering features like a shader pipeline or post-processing.
-   A GUI system.

### 7. System Actors
| Actor | Category | Description |
| :--- | :--- | :--- |
| **Application Developer** | Human | The primary user of the library who writes code to generate `RenderCommands`. |
| **CLI User** | Human | A user of the `are_cli` tool for interactive or headless rendering. |
| **Host Application** | External System | The program that consumes the `agnostic_renderer` crate. |
| **`unilang` Interpreter**| External System | The language interpreter that powers the `are_cli` tool. |

### 8. Functional Requirements: Scene Definition
#### 8.1. FR-A: Scene & Command Queue
- **FR-A1:** Must provide a `Scene` object as a container for a frame.
- **FR-A2:** `Scene` must be composed of an ordered list of `RenderCommand`s.
- **FR-A3:** `Scene` must have a method to add commands (e.g., `scene.add(...)`).
- **FR-A4:** Must provide a `RenderCommand` enum wrapping all primitive commands.
- **FR-A5:** All command structs must be POD (`Copy`, `Clone`, `Serialize`).
- **FR-A6:** `Scene` must be queryable (e.g., `scene.query_by_type<T>()`).

#### 8.2. FR-B: Core Rendering Primitives
- **FR-B1:** Must provide a `LineCommand` with `StrokeStyle`.
- **FR-B2:** Must provide a `CurveCommand` for Bezier curves.
- **FR-B3:** Must provide a `TextCommand` with `FontStyle` and `TextAnchor`.
- **FR-B4:** Must provide a `TilemapCommand`.
- **FR-B5:** Must provide a `ParticleEmitterCommand`.

### 9. Functional Requirements: CLI
- **FR-C1:** Must be built using the `unilang` crate.
- **FR-C2:** Must provide commands for scene management (`scene.new`, `scene.add`, `scene.list`).
- **FR-C3:** `scene.add` must support all core primitives.
- **FR-C4:** Must support loading and saving scenes from/to a file.
- **FR-C5:** Must provide a `render` command.
- **FR-C6:** `render` command must allow specifying the output file and backend adapter.
- **FR-C7:** Must be able to run in a fully headless environment.

### 10. Non-Functional Requirements
- **NFR-1:** **Performance:** Process 10,000 commands in under 16ms.
- **NFR-2:** **Performance:** Architecture must be designed for parallelism.
- **NFR-3:** **API Usability:** Core crate must have no dependency on any specific graphics backend.
- **NFR-4:** **API Usability:** Must adhere to Rust API Guidelines.
- **NFR-5:** **API Usability:** Must achieve 100% documentation coverage.
- **NFR-6:** **Reliability:** Adapters must use reference-based visual testing.
- **NFR-7:** **Extensibility:** Creating a new adapter must be straightforward and well-documented.
- **NFR-8:** **Compatibility:** Must support major desktop platforms and WASM.

---

**Part II: Internal Design (Backend Implementation)**

### 11. System Architecture (Ports & Adapters)
It is strongly recommended to use a Ports and Adapters architecture. The `agnostic_renderer` crate contains the core logic and `Port` traits. Separate `Adapter` crates implement these traits using specific backend technologies, ensuring the core is decoupled from implementation details.

### 12. External Dependencies Analysis
The choice of libraries for backend adapters involves trade-offs.
- **SVG:** The `svg` crate is a strong candidate due to its builder pattern API, which maps well to the `RenderCommand` model.
- **Terminal:** `crossterm` is a good choice for its raw terminal manipulation capabilities, while `ratatui` could be used for more structured TUI output if needed. The simpler `crossterm` is likely sufficient.

### 13. Core Rendering Ports (Traits)
- A primary `Renderer` trait should define the rendering lifecycle (`begin_frame`, `render_scene`, `end_frame`).
- The `render_scene` method will dispatch commands to a `PrimitiveRenderer` trait.
- The `Renderer` trait should include methods for runtime capability discovery (e.g., `supports_textures()`).

### 14. Required Backend Adapters
1.  **`renderer_adapter_wgpu`:** For high-performance native rendering.
2.  **`renderer_adapter_webgl`:** For web browser rendering via WASM.
3.  **`renderer_adapter_svg`:** For generating static vector graphics files.
4.  **`renderer_adapter_terminal`:** For headless rendering and rapid prototyping in the terminal.

### 15. Architectural & Flow Diagrams
*(Diagrams from Section 13 of the interactive session are considered part of the spec. Key diagrams include the C4 Container, Ports & Adapters Architecture, and Deployment Diagram.)*

---

**Part III: Project & Process Governance**

### 16. Core Principles of Development
1.  **Single Source of Truth:** The Git monorepo is absolute.
2.  **Documentation-First Development:** Spec changes are reviewed and merged before code.
3.  **Review-Driven Change Control:** All changes must go through a Pull Request.
4.  **Radical Transparency:** All decisions are captured in writing.
5.  **File Naming Conventions:** All files must use `snake_case`.

### 17. Open Questions
| ID | Question | Status |
| :--- | :--- | :--- |
| **Q1** | Optimal strategy for backend-agnostic asset loading (fonts, textures). | Open |
| **Q2** | Optimal `unilang` syntax for complex, nested parameters. | Open |
| **Q3** | `wgpu` adapter window management strategy (create window vs. use raw handle). | Open |

---

### Appendix: Addendum

#### Purpose
This document is intended to be completed by the **Developer** during the implementation phase to capture the final, as-built details of the **Internal Design**.

#### Conformance Checklist
*This checklist is the definitive list of acceptance criteria. Before final delivery, each item must be verified as complete and marked with `✅`.*

| Status | Requirement ID | Requirement Summary |
| :--- | :--- | :--- |
| ❌ | **FR-A1** | Must provide a `Scene` object. |
| ❌ | **FR-A2** | `Scene` is an ordered list of `RenderCommand`s. |
| ❌ | **FR-A3** | `Scene` has an `add` method. |
| ❌ | **FR-A4** | Must provide a `RenderCommand` enum. |
| ❌ | **FR-A5** | Command structs must be POD. |
| ❌ | **FR-A6** | `Scene` must be queryable. |
| ❌ | **FR-B1** | Must provide `LineCommand`. |
| ❌ | **FR-B2** | Must provide `CurveCommand`. |
| ❌ | **FR-B3** | Must provide `TextCommand`. |
| ❌ | **FR-B4** | Must provide `TilemapCommand`. |
| ❌ | **FR-B5** | Must provide `ParticleEmitterCommand`. |
| ❌ | **FR-C1** | CLI must use `unilang`. |
| ❌ | **FR-C2** | CLI must have scene management commands. |
| ❌ | **FR-C3** | CLI `add` must support all primitives. |
| ❌ | **FR-C4** | CLI must support load/save of scenes. |
| ❌ | **FR-C5** | CLI must have a `render` command. |
| ❌ | **FR-C6** | `render` must support output file and backend selection. |
| ❌ | **FR-C7** | CLI must support headless operation. |

#### Finalized Internal Design Decisions
- *To be completed by the developer.*

#### Finalized Internal Data Models
- *To be completed by the developer.*

#### Environment Variables
- *To be completed by the developer.*

#### Finalized Library & Tool Versions
- *To be completed by the developer.*

#### Deployment Checklist
- *To be completed by the developer.*
