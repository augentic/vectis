## Why

A code review of `examples/opsx_todo` (generated via the crux-app schema) found
10 Critical-severity logic bugs that the compiler, tests, and clippy cannot
catch. These bugs cause data loss under real-world usage patterns: rapid user
actions discard pending changes, SSE events corrupt in-flight sync state,
conflict resolution silently drops local edits for newly-created items, and
delete operations are sent to the server for items it has never seen.

Five mechanical issues (missing `render()`, input validation, serde derives,
unused dependency) have already been fixed. The remaining findings require
design-level changes to data types, event signatures, and sync logic.

## What Changes

- **BREAKING** Change `PendingOp::Delete(String)` to `Delete { id: String, deleted_at: String }` to carry conflict-resolution timestamps
- **BREAKING** Change `TodoItem.updated_at` from `Option<String>` to `String` to eliminate ambiguous conflict-resolution fallbacks
- **BREAKING** Change `Event::AddTodo` to accept `(id, timestamp)` from the shell instead of generating sequential IDs internally
- **BREAKING** Add timestamp parameters to `Event::EditTitle` and `Event::ToggleCompleted`
- Add pending-op coalescing logic to `DeleteTodo` and `ClearCompleted` — skip server deletes for items that only exist as pending Creates
- Fix `OpResponse` handler to remove only the specific synced op (by position/index), not all ops matching the item ID — prevents rapid-action data loss
- Fix SSE `item_deleted` and `apply_server_item` to respect `syncing_id` — do not remove pending ops for items with an in-flight sync
- Replace `unwrap_or_default()` on persisted-state deserialization with error handling that surfaces corruption to the user
- Add 7 missing edge-case tests covering the scenarios identified by the review

## Capabilities

### New Capabilities

_(none — this change modifies existing logic, no new capabilities)_

### Modified Capabilities

_(no spec-level changes — the app-spec.md already describes the correct behavior; the implementation simply did not match it)_

## Impact

- **`examples/opsx_todo/shared/src/app.rs`** — Primary file affected. All domain types, event handlers, helper functions, and tests are modified.
- **Shell code** — Any shell code sending `Event::AddTodo`, `Event::EditTitle`, or `Event::ToggleCompleted` must be updated to pass ID/timestamp parameters. Since no shell exists yet for opsx_todo, this is a documentation-only impact.
- **Persisted state migration** — The `PersistedState` struct changes shape (`PendingOp::Delete` gains a field, `TodoItem.updated_at` loses `Option`, `next_local_id` is removed). Existing persisted data will fail to deserialize. The fix should handle this gracefully (log and reset).
