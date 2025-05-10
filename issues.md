# Code Issues and Tasks

This document lists task comments found in the codebase, formatted for creating GitHub issues.

---

## Issue: Placeholder comment in 2D tests module

*   **Location:** `module/math/ndarray_cg/tests/inc/d2_test/mod.rs`, Line 10
*   **Original Comment:** `// xxx`
*   **Context/Elaboration:** A brief placeholder comment in the 2D tests module. This suggests an incomplete thought or a placeholder for future work related to this module or its tests.
*   **Action Needed:** Clarify the purpose of this placeholder comment or remove it if no longer needed.

---

## Issue: Fix tests in ndarray_cg indexing_test module

*   **Location:** `module/math/ndarray_cg/tests/inc/d2_test/access_test/mod.rs`, Line 12
*   **Original Comment:** `// qqq : fix tests, please`
*   **Context/Elaboration:** This task comment suggests that the tests within the `indexing_test` module need to be fixed.
*   **Action Needed:** Investigate and fix any failing tests within the `ndarray_cg/tests/inc/d2_test/access_test/indexing_test` module.

---

## Issue: Remove unnecessary hset import in ndarray_cg iter_test

*   **Location:** `module/math/ndarray_cg/tests/inc/d2_test/access_test/indexing_test/iter_test.rs`, Line 3
*   **Original Comment:** `use test_tools::hset; // xxx : remove it later`
*   **Context/Elaboration:** This task comment suggests that the import of `hset` from `test_tools` should be removed later, implying it might be a temporary or unused import.
*   **Action Needed:** Remove the `use test_tools::hset;` line if it is no longer necessary.

---

## Issue: Placeholder comment in ndarray_cg matrix addition tests

*   **Location:** `module/math/ndarray_cg/tests/inc/d2_test/arithmetic_test/add_test.rs`, Line 47
*   **Original Comment:** `// xxx`
*   **Context/Elaboration:** A brief placeholder comment in the matrix addition tests. This suggests an incomplete thought or a placeholder for future work related to these addition tests.
*   **Action Needed:** Clarify the purpose of this placeholder comment or remove it if no longer needed.

---

## Issue: Uncomment matrix multiplication test in ndarray_cg

*   **Location:** `module/math/ndarray_cg/tests/inc/d2_test/arithmetic_test/mul_test.rs`, Line 70
*   **Original Comment:** `// xxx : uncomment`
*   **Context/Elaboration:** This task comment suggests uncommenting the test below it, indicating that the test is intentionally commented out and should be re-enabled later.
*   **Action Needed:** Uncomment the test function starting at line 71.

---

## Issue: Implement trybuild test for incompatible matrix multiplication

*   **Location:** `module/math/ndarray_cg/tests/inc/d2_test/arithmetic_test/mul_test.rs`, Line 78
*   **Original Comment:** `// // qqq : implement try build test throwing error`
*   **Context/Elaboration:** This task comment suggests implementing a `trybuild` test for incompatible dimensions multiplication, which would verify that the code correctly produces a compile-time error in such cases.
*   **Action Needed:** Implement a `trybuild` test to verify compile-time errors for incompatible matrix multiplication.

---

## Issue: Consider usize for VectorDataType natoms in mingl

*   **Location:** `module/min/mingl/src/data_type.rs`, Line 46
*   **Original Comment:** `// xxx : usize?`
*   **Context/Elaboration:** This task comment suggests considering if the type for `natoms` in the `VectorDataType` struct should be `usize` instead of `i32` for better type safety and representation of size/count.
*   **Action Needed:** Evaluate if changing the type of `natoms` in `VectorDataType` from `i32` to `usize` is appropriate and implement the change if necessary.

---

## Issue: Consider usize for natoms method return type in mingl

*   **Location:** `module/min/mingl/src/data_type.rs`, Line 67
*   **Original Comment:** `// xxx : usize?`
*   **Context/Elaboration:** This task comment suggests considering if the return type of the `natoms` method should be `usize` instead of `i32`.
*   **Action Needed:** Evaluate if changing the return type of the `natoms` method from `i32` to `usize` is appropriate and implement the change if necessary.

---

## Issue: Verify nelements method logic in mingl

*   **Location:** `module/min/mingl/src/data_type.rs`, Line 80
*   **Original Comment:** `// xxx : qqq : verify`
*   **Context/Elaboration:** This task comment suggests verifying the logic of the `nelements` method in the `VectorDataType` struct to ensure it correctly calculates the number of elements.
*   **Action Needed:** Review and verify the logic of the `nelements` method in `mingl/src/data_type.rs`.

---

## Issue: Make exposed use ::former unnecessary in mingl derive module

*   **Location:** `module/min/mingl/src/derive.rs`, Line 12
*   **Original Comment:** `exposed use ::former; // xxx : make it unncecessary`
*   **Context/Elaboration:** This task comment suggests that the explicit `exposed use ::former;` in the `mod_interface!` block is unnecessary and should be removed or refactored to achieve the desired module exports implicitly or through a different mechanism.
*   **Action Needed:** Refactor the `mingl/src/derive.rs` module to make the `exposed use ::former;` line unnecessary.

---

## Issue: Replace bytemuck dependency in minwebgl

*   **Location:** `module/min/minwebgl/Cargo.toml`, Line 65
*   **Original Comment:** `# bytemuck = { workspace = true, optional = true, features = [ "derive" ] } # xxx : replace`
*   **Context/Elaboration:** This task comment suggests that the `bytemuck` dependency should be replaced with an alternative in the `minwebgl` crate's `Cargo.toml`.
*   **Action Needed:** Identify a suitable replacement for the `bytemuck` dependency and update the `minwebgl/Cargo.toml` and any relevant code accordingly.

---

## Issue: Investigate browser_log reuse in minwebgl browser module

*   **Location:** `module/min/minwebgl/src/browser.rs`, Line 10
*   **Original Comment:** `// xxx : investigate`
*   **Context/Elaboration:** This task comment suggests that something related to the reuse of `browser_log` in the `minwebgl::browser` module needs investigation. The specific issue is not detailed, requiring further analysis of the code and its interaction with `browser_log`.
*   **Action Needed:** Investigate the reuse of `browser_log` in `minwebgl/src/browser.rs` to understand the context and identify any potential issues or areas for improvement.

---

## Issue: Refactor geometry attribute handling in minwebgl

*   **Location:** `module/min/minwebgl/src/geometry.rs`, Line 53
*   **Original Comment:** `// qqq : xxx : move out switch and make it working for all types`
*   **Context/Elaboration:** This task comment suggests that a `match` statement used for handling different attribute sizes/types in the geometry module should be refactored into a more generic mechanism that can handle all supported types without explicit matching.
*   **Action Needed:** Refactor the attribute handling logic in `minwebgl/src/geometry.rs` to be more generic and support all relevant types.

---

## Issue: Clean up commented-out draw method in minwebgl ProgramInterface

*   **Location:** `module/min/minwebgl/src/shader.rs`, Line 214
*   **Original Comment:** `// xxx : clean`
*   **Context/Elaboration:** This task comment suggests cleaning up the commented-out `draw` method in the `ProgramInterface` trait.
*   **Action Needed:** Review the commented-out `draw` method in the `ProgramInterface` trait and either remove it or uncomment and integrate it if it's still needed.

---

## Issue: Clean up commented-out draw method in minwebgl Program struct

*   **Location:** `module/min/minwebgl/src/shader.rs`, Line 373
*   **Original Comment:** `// xxx : clean`
*   **Context/Elaboration:** This task comment suggests cleaning up the commented-out `draw` method in the `Program` struct implementation.
*   **Action Needed:** Review the commented-out `draw` method in the `Program` struct implementation and either remove it or uncomment and integrate it if it's still needed.