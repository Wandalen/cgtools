# Feature Doc Definition

A **feature** instance documents one cohesive slice of the crate's public API. In `minvulkan`, each instance covers one capability that removes raw Vulkan/`ash` setup boilerplate without hiding the underlying Vulkan objects — the same convention [`minwgpu`'s feature docs](../../../minwgpu/docs/feature/readme.md) use. The table below is the index into them.

### Scope

- **Purpose**: `minvulkan`'s API exists to remove `ash`/Vulkan setup boilerplate without hiding the underlying Vulkan objects.
- **Responsibility**: Document each such capability as a navigational hub over its source and tests.
- **In Scope**: Native Vulkan instance/device construction, window surface and swapchain presentation, and any further capability `minvulkan` grows.
- **Out of Scope**: Implementation-level Vulkan/`ash` descriptor detail (see the Sources references inside each instance).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Native Context and Device](001_native_context_and_device.md) | Vulkan instance/physical-device/logical-device/queue setup, mirroring `minwgpu`'s context builder | ✅ |
| 002 | [Window Surface and Swapchain](002_window_surface_and_swapchain.md) | `VkSurfaceKHR` from raw handle traits plus a real `VK_KHR_swapchain` with a per-frame acquire/present pair | ✅ |
