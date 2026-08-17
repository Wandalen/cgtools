# Feature Doc Definition

A **feature** instance documents one cohesive slice of the crate's public API. In `minvulkan`, each instance will cover one builder-based capability that removes raw Vulkan/`ash` setup boilerplate without hiding the underlying Vulkan objects — the same convention [`minwgpu`'s feature docs](../../../minwgpu/docs/feature/readme.md) use. This collection currently holds one committed-but-not-yet-elaborated instance; the table below is the index into it.

### Scope

- **Purpose**: `minvulkan`'s builders will exist to remove `ash`/Vulkan setup and resource-construction boilerplate without hiding the underlying Vulkan objects.
- **Responsibility**: Document each builder-based capability as a navigational hub over its source and tests, once implemented.
- **In Scope**: Native Vulkan instance/device/surface construction and any further builder-based capability `minvulkan` grows.
- **Out of Scope**: Implementation-level Vulkan/`ash` descriptor detail (see the Sources references inside each instance, once written).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Native Context and Device](001_native_context_and_device.md) | Vulkan instance/physical-device/logical-device/queue setup, mirroring `minwgpu`'s context builder | ✅ |
