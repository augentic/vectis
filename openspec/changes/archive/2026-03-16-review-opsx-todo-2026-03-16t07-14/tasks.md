## 1. Data Model Hardening

- [x] 1.1 Add `deleted_at: DateTime<Utc>` to `PendingOp::Delete` variant
- [x] 1.2 Update `apply_server_item` LWW to compare `deleted_at` for delete conflicts
- [x] 1.3 Guard SSE `item_deleted` handler: preserve pending ops when `syncing_id` matches
- [x] 1.4 Replace `unwrap_or_default()` on corrupted state with `tracing::warn!` + error surface
- [x] 1.5 Handle `StateSaved(Err(_))` -- surface as warning or retry
- [x] 1.6 Add tests for corrupted-state recovery and save-failure handling

## 2. Operation Coalescing

- [x] 2.1 In `DeleteTodo`, check for Create-only items and eliminate without network request
- [x] 2.2 In `ClearCompleted`, apply same Create-then-Delete elimination
- [x] 2.3 Coalesce duplicate `PendingOp::Update` entries for the same item
- [x] 2.4 Add tests for Create-then-Delete coalescing and rapid-toggle deduplication

## 3. Determinism & Observability

- [x] 3.1 Replace `model.next_id` with shell-provided UUID in `AddTodo` event
- [x] 3.2 Replace `Utc::now()` calls with `crux_time` capability requests
- [x] 3.3 Change `active_count` from `String` to `usize` in `TodoListView`
- [x] 3.4 Change `sync_status` from `String` to `SyncStatus` enum in `TodoListView`
- [x] 3.5 Add edge-case tests: SSE-during-sync, conflict resolution, ClearCompleted-empty

## 4. Verification

- [x] 4.1 `cargo check`
- [x] 4.2 `cargo test` -- all existing + new tests pass
- [x] 4.3 `cargo clippy --all-targets` -- zero warnings
