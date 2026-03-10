## ADDED Requirements

### Requirement: Pending-op coalescing on delete

The system SHALL skip sending a DELETE to the server when the deleted item
only exists as a pending Create that has never been synced.

#### Scenario: Delete item that was never synced

- **WHEN** user creates item A (pending Create queued) and deletes item A before sync runs
- **THEN** the pending Create for A is removed and no pending Delete is queued

#### Scenario: Delete item that was synced then updated locally

- **WHEN** user updates item A (pending Update queued after a previous successful sync) and deletes item A
- **THEN** all pending ops for A are removed and a pending Delete with a `deleted_at` timestamp is queued

#### Scenario: ClearCompleted coalesces pending Creates

- **WHEN** user creates item A, marks it completed, and taps Clear Completed — all before sync
- **THEN** the pending Create for A is removed and no pending Delete is queued for A

---

### Requirement: SSE handlers respect in-flight sync

The system SHALL NOT remove pending ops for an item that has an in-flight sync
operation (`syncing_id` matches the item ID) when processing an SSE event.

#### Scenario: SSE item_deleted during in-flight sync for same item

- **WHEN** sync is in-flight for item A and an SSE `item_deleted` event arrives for item A
- **THEN** item A is removed from the local item list but its pending ops are preserved until the sync response arrives

#### Scenario: SSE item_updated during in-flight sync for same item

- **WHEN** sync is in-flight for item A and an SSE `item_updated` event arrives for item A
- **THEN** the server item is stored for later conflict resolution but pending ops for A are not removed

---

### Requirement: Sync response removes only the synced op

The system SHALL remove only the specific pending op that was synced (the first
op in the queue) when processing a successful sync response — not all ops
matching the synced item ID.

#### Scenario: Rapid toggle preserves second change

- **WHEN** user toggles item A, sync starts, user toggles item A again before sync completes, and the first sync responds successfully
- **THEN** the first pending Update is removed, the second pending Update (with the latest state) remains in the queue, and a new sync starts for it

#### Scenario: Edit during in-flight sync preserves edit

- **WHEN** user edits item A's title, sync starts, user edits item A's title again, and the first sync responds
- **THEN** the second edit's pending Update remains in the queue and is synced next

---

### Requirement: Deterministic event parameters

All mutating Event variants SHALL accept IDs and timestamps as parameters from
the shell. The core SHALL NOT generate IDs or timestamps internally.

#### Scenario: AddTodo receives ID and timestamp

- **WHEN** shell sends AddTodo with id="cuid_abc" and timestamp="2025-06-15T10:30:00Z"
- **THEN** the created TodoItem has id="cuid_abc" and updated_at="2025-06-15T10:30:00Z"

#### Scenario: EditTitle receives timestamp

- **WHEN** shell sends EditTitle with id, new_title, and timestamp
- **THEN** the updated TodoItem has the provided timestamp as its updated_at

---

### Requirement: Non-optional conflict-resolution timestamps

`TodoItem.updated_at` SHALL be a non-optional `String`. `PendingOp::Delete`
SHALL carry a `deleted_at: String` timestamp.

#### Scenario: Conflict resolution with non-optional timestamps

- **WHEN** an SSE update arrives for item A with server timestamp "2025-06-15T12:00:00Z" and the local item has updated_at "2025-06-15T11:00:00Z"
- **THEN** the server version wins (server timestamp is newer) and pending ops for A are removed

#### Scenario: Local-wins conflict resolution

- **WHEN** an SSE update arrives for item A with server timestamp "2025-06-15T10:00:00Z" and the local item has updated_at "2025-06-15T11:00:00Z" with a pending Update
- **THEN** the local version is preserved and the pending Update remains in the queue

---

### Requirement: Graceful persisted-state deserialization

The system SHALL handle corrupted or schema-mismatched persisted state by
logging the error and starting with empty state, rather than silently replacing
data via `unwrap_or_default()`.

#### Scenario: Corrupted persisted bytes

- **WHEN** the app loads and the persisted bytes fail to deserialize
- **THEN** the error is logged, the app transitions to the TodoList page with empty state, and no data is silently discarded
