# tests

Integration tests for `behaviour_tree`, exercising the crate's public API only
(relocated from the inline `#[ cfg( test ) ]` module by task 067). Includes the
fully documented `RepeatNode::infinite` livelock-guard bug reproducer, which runs
the risky `execute()` on a background thread with a bounded `recv_timeout`.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| behaviour_tree_test.rs | Context state, composite/decorator semantics, builder, livelock guard |
