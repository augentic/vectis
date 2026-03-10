## 1. Data Type Changes

- [ ] 1.1 Change `TodoItem.updated_at` from `Option<String>` to `String` in `shared/src/app.rs`
- [ ] 1.2 Change `PendingOp::Delete(String)` to `PendingOp::Delete { id: String, deleted_at: String }` and update `item_id()` match arm
- [ ] 1.3 Remove `next_local_id: u64` from `Model` and `PersistedState`
- [ ] 1.4 Update all code referencing the changed types — constructors, pattern matches, field access — until `cargo check` passes

## 2. Event Signature Changes

- [ ] 2.1 Change `Event::AddTodo` to `Event::AddTodo(String, String)` (id, timestamp) and update the handler to use the provided values instead of the counter
- [ ] 2.2 Change `Event::EditTitle(String, String)` to `Event::EditTitle(String, String, String)` (id, new_title, timestamp) and set `item.updated_at` from the timestamp
- [ ] 2.3 Change `Event::ToggleCompleted(String)` to `Event::ToggleCompleted(String, String)` (id, timestamp) and set `item.updated_at` from the timestamp
- [ ] 2.4 Change `Event::DeleteTodo(String)` to `Event::DeleteTodo(String, String)` (id, timestamp) and use the timestamp for `PendingOp::Delete { deleted_at }`
- [ ] 2.5 Change `Event::ClearCompleted` to `Event::ClearCompleted(String)` (timestamp) and use the timestamp for each `PendingOp::Delete { deleted_at }`

## 3. Pending-Op Coalescing

- [ ] 3.1 In the `DeleteTodo` handler, before pushing a `PendingOp::Delete`, inspect the removed ops: if only `Create` ops existed for the item, skip the Delete entirely
- [ ] 3.2 In the `ClearCompleted` handler, apply the same coalescing logic per-item in the loop (reference `examples/todo` for the `saw_create`/`saw_non_create` pattern)

## 4. Sync Response Fix

- [ ] 4.1 In the `OpResponse(Ok)` handler, replace `pending_ops.retain(|op| op.item_id() != synced_id)` with removal of only the first element (`pending_ops.remove(0)`) after verifying it matches `synced_id`
- [ ] 4.2 Apply the same fix to the `DeleteOpResponse(Ok)` handler

## 5. SSE Handler Fixes

- [ ] 5.1 In the `item_deleted` SSE handler, check `model.syncing_id` before calling `pending_ops.retain()` — skip removing ops if `syncing_id` matches the deleted item's ID
- [ ] 5.2 In `apply_server_item`, check `model.syncing_id` before calling `pending_ops.retain()` — skip removing ops if `syncing_id` matches the server item's ID

## 6. Conflict Resolution Cleanup

- [ ] 6.1 Update `apply_server_item` and `merge_server_items` conflict-resolution logic to use direct `String` comparison on `updated_at` (no more `Option` matching with `_ => true` fallback)

## 7. Persisted-State Deserialization

- [ ] 7.1 Replace `serde_json::from_slice(&bytes).unwrap_or_default()` with a `match` that logs the deserialization error and continues with empty state

## 8. Update Existing Tests

- [ ] 8.1 Update all existing test call sites for the changed Event signatures (AddTodo, EditTitle, ToggleCompleted, DeleteTodo, ClearCompleted) — supply test IDs and timestamps
- [ ] 8.2 Update test helper `make_item` to use non-optional `updated_at`
- [ ] 8.3 Run `cargo test` to confirm all existing tests pass with the new signatures

## 9. Add Missing Edge-Case Tests

- [ ] 9.1 Add test: SSE `item_updated` arrives during in-flight sync for the same item — pending ops preserved
- [ ] 9.2 Add test: SSE `item_deleted` arrives during in-flight sync — pending ops not clobbered
- [ ] 9.3 Add test: `EditTitle` with empty/whitespace-only string is a no-op
- [ ] 9.4 Add test: conflict resolution where local has newer timestamp than server — local wins
- [ ] 9.5 Add test: `ClearCompleted` coalesces pending Creates (no server delete queued)
- [ ] 9.6 Add test: rapid toggle of same item — second pending Update survives first sync response
- [ ] 9.7 Add test: delete of item that was never synced (Create→Delete before sync) — no Delete op queued

## 10. Verification

- [ ] 10.1 Run `cargo check` in `examples/opsx_todo`
- [ ] 10.2 Run `cargo test` in `examples/opsx_todo`
- [ ] 10.3 Run `cargo clippy --all-targets` in `examples/opsx_todo`
- [ ] 10.4 Run the code-reviewer skill against `examples/opsx_todo` to confirm all Critical findings are resolved
