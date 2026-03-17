# iOS Shell Specification: opsx_todo

## Overview

SwiftUI shell for the `examples/opsx_todo` Crux todo-list application with
offline-first sync, real-time SSE updates, and filter controls.

## Target Directory

`examples/opsx_todo/iOS`

## Navigation Style

**single** — the app has one route (`Route::TodoList`) with loading and error
states handled as full-screen overlays within the same navigation context.

ViewModel mapping:

| ViewModel variant | Screen |
|---|---|
| `Loading` | Centered `ProgressView` with "Loading..." label |
| `Error(ErrorView)` | Centered error message with retry button |
| `TodoList(TodoListView)` | Main list with input field, items, filters, footer |

## Screen Customizations

### Loading

A simple centered `ProgressView`. No additional customizations.

### Error

- Display `ErrorView.message` in body text
- Show "Retry" button when `ErrorView.can_retry` is true
- Retry sends `Event::Navigate(Route::TodoList)`

### TodoList

#### Input Area (top)
- `TextField` bound to `TodoListView.new_title` via `Event::SetNewTitle(String)`
- Submit on return key: generate a UUID and send `Event::AddTodo { id }`
- Disable add button when `new_title` is empty after trimming

#### Item List
- Each row shows `TodoItemView.title` with a leading checkmark toggle
- Tap checkmark: `Event::ToggleTodo(id)`
- Swipe-to-delete: `Event::DeleteTodo(id)`
- Tap row to edit title inline: `Event::EditTitle { id, title }`
- Strikethrough styling on completed items

#### Filter Bar
- Segmented control with `All` / `Active` / `Completed` matching `Filter` enum
- Sends `Event::SetFilter(filter)`

#### Footer
- Left: active count from `TodoListView.active_count` (format as
  "{n} item(s) left" in the shell)
- Right: "Clear completed" button, visible when `TodoListView.has_completed`
  is true, sends `Event::ClearCompleted`

#### Status Bar (bottom)
- Sync indicator derived from `TodoListView.sync_status` and
  `TodoListView.sse_state`:
  - `Idle` + `Connected` → "Synced" (green dot)
  - `Syncing` → "Syncing..." (animated indicator)
  - `Offline` → "Offline" (red dot)
  - `Idle` + `Connecting` → "Connecting..." (orange dot)
  - `Idle` + `Disconnected` → "Disconnected" (gray dot)
- Pending count badge: show `TodoListView.pending_count` when > 0

## Platform Features

| Feature | Details |
|---|---|
| Haptic feedback | Light impact on toggle, medium on delete |
| Pull-to-refresh | Not applicable (SSE provides real-time updates) |

## Design System Overrides

None — use default VectisDesign tokens.
