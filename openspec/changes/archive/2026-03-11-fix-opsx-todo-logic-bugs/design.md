## Context

The `examples/opsx_todo` app was generated via the crux-app schema and
core-writer skill as a validation exercise. A code review using the
code-reviewer skill found that while the generated code compiles and passes
basic tests, it contains logic bugs in the sync/conflict-resolution layer
that cause data loss under concurrent or rapid-action scenarios.

The reference implementation at `examples/todo` handles these scenarios
correctly. This change brings `opsx_todo` to parity with the reference by
fixing the underlying data structures and event-handling logic.

All changes are confined to `examples/opsx_todo/shared/src/app.rs` and its
types. No new crates or capabilities are introduced.

## Goals / Non-Goals

**Goals:**

- Eliminate all Critical-severity findings from the code review (CRX-003,
  CRX-004, LOG-003, LOG-004, LOG-006)
- Make the core deterministic and fully testable by accepting IDs and
  timestamps from the shell (CRX-007)
- Add edge-case tests for every identified risk scenario (LOG-008)
- Handle persisted-state deserialization failures gracefully (LOG-005)

**Non-Goals:**

- Rewriting the app from scratch — changes are surgical fixes to the existing
  generated code
- Changing the app spec — the spec already describes correct behavior; only
  the implementation needs fixing
- Adding new features or capabilities
- Changing ViewModel field types from String to typed values (CRX-008, Info
  severity — deferred)

## Decisions

### 1. Make `updated_at` non-optional on `TodoItem`

**Decision**: Change `updated_at: Option<String>` to `updated_at: String`.

**Rationale**: Every `TodoItem` must have a timestamp for last-writer-wins
conflict resolution to work. When `updated_at` is `None`, the current code
falls through to `_ => true` (server always wins), silently discarding valid
local edits on newly-created items.

**Alternative considered**: Keep `Option` and change the fallback to "local
wins when server timestamp is absent." Rejected because it only shifts the
data-loss risk to the other direction. A non-optional field eliminates the
ambiguity entirely. The shell provides the timestamp at creation time.

### 2. Add `deleted_at` timestamp to `PendingOp::Delete`

**Decision**: Change `Delete(String)` to `Delete { id: String, deleted_at: String }`.

**Rationale**: Without a timestamp, the conflict-resolution code cannot
determine whether a local delete happened before or after a server update.
The `deleted_at` field enables the same last-writer-wins comparison used
for Create and Update ops.

**Alternative considered**: Use the item's last `updated_at` from the model.
Rejected because the item is removed from `model.items` on delete, so its
timestamp is no longer available when the SSE conflict handler runs.

### 3. Accept ID and timestamp from the shell on mutating Events

**Decision**: Change event signatures:
- `AddTodo` → `AddTodo(String, String)` — (id, timestamp)
- `EditTitle(String, String)` → `EditTitle(String, String, String)` — (id, new_title, timestamp)
- `ToggleCompleted(String)` → `ToggleCompleted(String, String)` — (id, timestamp)
- `DeleteTodo(String)` → `DeleteTodo(String, String)` — (id, timestamp)
- `ClearCompleted` → `ClearCompleted(String)` — (timestamp)

**Rationale**: Keeps the Crux core deterministic. Tests supply known values
without mocking clocks or random generators. Matches the reference `todo` app.
Removes the `next_local_id` counter from the model.

### 4. Remove only the first matching op in `OpResponse`, not all ops for the item

**Decision**: In the `OpResponse(Ok)` and `DeleteOpResponse(Ok)` handlers,
remove only the op at index 0 of `pending_ops` (the one that was synced)
rather than using `retain(|op| op.item_id() != synced_id)`.

**Rationale**: When the user modifies the same item twice before the first
sync completes, the second op is queued behind the first. The current
`retain`-by-item-id approach removes both ops when the first sync responds,
silently discarding the user's latest change. Removing only the first op
preserves the second for the next sync cycle.

**Precondition**: `start_sync` always picks `pending_ops[0]`, so the synced
op is always the first one. This invariant is already maintained.

### 5. SSE handlers respect `syncing_id`

**Decision**: In `item_deleted` and `apply_server_item`, before calling
`pending_ops.retain(...)`, check whether `model.syncing_id` matches the
affected item. If it does, skip removing pending ops for that item — the
in-flight sync response handler will clean them up.

**Rationale**: Without this guard, an SSE event arriving during an in-flight
sync removes the op that the sync is processing. When the sync response
arrives, the response handler finds nothing to clean up, and any subsequent
pending op for the same item (added between the SSE event and the sync
response) was also removed by the overly broad `retain`.

### 6. Add coalescing logic to `DeleteTodo` and `ClearCompleted`

**Decision**: Before pushing a `PendingOp::Delete`, inspect the ops being
removed. If the only ops for that item were `Create` ops (the item was never
synced), skip the `Delete` entirely.

**Rationale**: Sending a DELETE to the server for an item it has never seen
results in a 404 error, which triggers the offline error-handling path and
confuses sync state.

### 7. Handle persisted-state deserialization failure

**Decision**: Replace `serde_json::from_slice(&bytes).unwrap_or_default()`
with a match that logs the error and transitions to `Page::TodoList` with
an empty state, rather than silently discarding data.

**Rationale**: If the persisted bytes are corrupted (e.g., schema change
from this very PR), `unwrap_or_default()` silently replaces all user data
with an empty state. Logging makes the data loss visible for debugging.

## Risks / Trade-offs

- **Persisted-state migration** — The `PersistedState` struct shape changes
  (`Delete` gains a field, `updated_at` loses `Option`, `next_local_id` is
  removed). Existing persisted data from previous runs will fail to
  deserialize. → Mitigated by Decision 7: the failure is logged and the app
  starts with empty state rather than crashing.

- **Shell contract changes** — All mutating Event variants gain parameters.
  Any future shell code must supply IDs and timestamps. → Low risk since no
  shell code exists for `opsx_todo` yet. The event changes are documented in
  the updated spec.

- **Index-based op removal** — Decision 4 relies on the invariant that
  `start_sync` always syncs `pending_ops[0]`. If this invariant is violated
  in the future, the wrong op could be removed. → Mitigated by adding a test
  that verifies the invariant and by documenting it with a comment.
