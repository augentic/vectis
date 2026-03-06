use crux_core::{
    macros::effect,
    render::{render, RenderOperation},
    App, Command,
};
use crux_http::HttpRequest;
use crux_kv::{error::KeyValueError, KeyValueOperation};
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::sse::{ServerSentEvents, SseMessage, SseRequest};

const API_URL: &str = "https://api.example.com";

type Http = crux_http::Http<Effect, Event>;
type KeyValue = crux_kv::KeyValue<Effect, Event>;

// ── Domain types ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TodoCreateBody {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TodoUpdateBody {
    pub title: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PendingOp {
    Create(TodoItem),
    Update(TodoItem),
    Delete { id: String, deleted_at: String },
}

impl PendingOp {
    fn item_id(&self) -> &str {
        match self {
            Self::Create(item) | Self::Update(item) => &item.id,
            Self::Delete { id, .. } => id,
        }
    }
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub enum SyncStatus {
    #[default]
    Idle,
    Syncing,
    Offline,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub enum SseConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct PersistedState {
    items: Vec<TodoItem>,
    pending_ops: Vec<PendingOp>,
}

#[derive(Deserialize)]
struct DeletedItemPayload {
    id: String,
}

// ── View model ──────────────────────────────────────────────────────────

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct TodoItemView {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct ViewModel {
    pub items: Vec<TodoItemView>,
    pub input_text: String,
    pub active_count: String,
    pub pending_count: usize,
    pub sync_status: String,
    pub filter: Filter,
}

// ── Model ───────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Model {
    items: Vec<TodoItem>,
    pending_ops: Vec<PendingOp>,
    filter: Filter,
    sync_status: SyncStatus,
    sse_state: SseConnectionState,
    input_text: String,
}

// ── Effects ─────────────────────────────────────────────────────────────

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
    KeyValue(KeyValueOperation),
    ServerSentEvents(SseRequest),
}

// ── Events ──────────────────────────────────────────────────────────────

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Event {
    // Shell-facing
    Initialize,
    SetInput(String),
    AddTodo(String, String),
    EditTitle(String, String, String),
    ToggleCompleted(String, String),
    DeleteTodo(String, String),
    SetFilter(Filter),
    RetrySync,
    ConnectSse,
    SseDisconnected,

    // Internal – KV
    #[serde(skip)]
    #[facet(skip)]
    DataLoaded(#[facet(opaque)] Result<Option<Vec<u8>>, KeyValueError>),

    #[serde(skip)]
    #[facet(skip)]
    DataSaved(#[facet(opaque)] Result<Option<Vec<u8>>, KeyValueError>),

    // Internal – HTTP
    #[serde(skip)]
    #[facet(skip)]
    ItemsFetched(#[facet(opaque)] crux_http::Result<crux_http::Response<Vec<TodoItem>>>),

    #[serde(skip)]
    #[facet(skip)]
    OpResponse(#[facet(opaque)] crux_http::Result<crux_http::Response<TodoItem>>),

    #[serde(skip)]
    #[facet(skip)]
    DeleteOpResponse(#[facet(opaque)] crux_http::Result<crux_http::Response<String>>),

    // Internal – SSE
    #[serde(skip)]
    #[facet(skip)]
    SseReceived(#[facet(opaque)] SseMessage),
}

// ── App ─────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct TodoApp;

impl TodoApp {
    fn save_state(model: &Model) -> Command<Effect, Event> {
        let state = PersistedState {
            items: model.items.clone(),
            pending_ops: model.pending_ops.clone(),
        };
        let bytes = serde_json::to_vec(&state).unwrap_or_default();
        KeyValue::set("todo_state", bytes).then_send(Event::DataSaved)
    }

    fn start_sync(model: &mut Model) -> Command<Effect, Event> {
        if model.pending_ops.is_empty() {
            model.sync_status = SyncStatus::Idle;
            return render();
        }
        if model.sync_status == SyncStatus::Syncing {
            return Command::done();
        }
        model.sync_status = SyncStatus::Syncing;

        let op = model.pending_ops[0].clone();
        let http_cmd = match op {
            PendingOp::Create(ref item) => {
                let body = TodoCreateBody {
                    id: item.id.clone(),
                    title: item.title.clone(),
                    completed: item.completed,
                };
                Http::post(format!("{API_URL}/api/todos"))
                    .body_json(&body)
                    .expect("serialize create body")
                    .expect_json()
                    .build()
                    .then_send(Event::OpResponse)
            }
            PendingOp::Update(ref item) => {
                let body = TodoUpdateBody {
                    title: item.title.clone(),
                    completed: item.completed,
                };
                Http::put(format!("{API_URL}/api/todos/{}", item.id))
                    .body_json(&body)
                    .expect("serialize update body")
                    .expect_json()
                    .build()
                    .then_send(Event::OpResponse)
            }
            PendingOp::Delete { ref id, .. } => Http::delete(format!("{API_URL}/api/todos/{id}"))
                .expect_string()
                .build()
                .then_send(Event::DeleteOpResponse),
        };

        render().and(http_cmd)
    }
}

impl App for TodoApp {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    #[allow(clippy::too_many_lines)]
    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            Event::Initialize => KeyValue::get("todo_state").then_send(Event::DataLoaded),

            Event::SetInput(text) => {
                model.input_text = text;
                render()
            }

            Event::AddTodo(id, timestamp) => {
                let title = model.input_text.trim().to_string();
                if title.is_empty() {
                    return Command::done();
                }
                let item = TodoItem {
                    id,
                    title,
                    completed: false,
                    updated_at: timestamp,
                };
                model.pending_ops.push(PendingOp::Create(item.clone()));
                model.items.push(item);
                model.input_text.clear();

                Self::save_state(model).and(Self::start_sync(model))
            }

            Event::EditTitle(id, new_title, timestamp) => {
                let new_title = new_title.trim().to_string();
                if new_title.is_empty() {
                    return Command::done();
                }
                if let Some(item) = model.items.iter_mut().find(|i| i.id == id) {
                    item.title = new_title;
                    item.updated_at = timestamp;
                    model.pending_ops.push(PendingOp::Update(item.clone()));
                }
                Self::save_state(model).and(Self::start_sync(model))
            }

            Event::ToggleCompleted(id, timestamp) => {
                if let Some(item) = model.items.iter_mut().find(|i| i.id == id) {
                    item.completed = !item.completed;
                    item.updated_at = timestamp;
                    model.pending_ops.push(PendingOp::Update(item.clone()));
                }
                Self::save_state(model).and(Self::start_sync(model))
            }

            Event::DeleteTodo(id, timestamp) => {
                model.items.retain(|i| i.id != id);
                model.pending_ops.push(PendingOp::Delete {
                    id,
                    deleted_at: timestamp,
                });
                Self::save_state(model).and(Self::start_sync(model))
            }

            Event::SetFilter(filter) => {
                model.filter = filter;
                render()
            }

            Event::RetrySync => Self::start_sync(model),

            Event::ConnectSse => {
                model.sse_state = SseConnectionState::Connecting;
                render().and(
                    ServerSentEvents::get_events(format!("{API_URL}/api/todos/events"))
                        .then_send(Event::SseReceived),
                )
            }

            Event::SseDisconnected => {
                model.sse_state = SseConnectionState::Disconnected;
                render().and(
                    Http::get(format!("{API_URL}/api/todos"))
                        .expect_json()
                        .build()
                        .then_send(Event::ItemsFetched),
                )
            }

            // ── Internal: KV ────────────────────────────────────────

            Event::DataLoaded(Ok(Some(bytes))) => {
                let state: PersistedState =
                    serde_json::from_slice(&bytes).unwrap_or_default();
                model.items = state.items;
                model.pending_ops = state.pending_ops;
                Command::all([
                    render(),
                    Command::event(Event::ConnectSse),
                    Command::event(Event::RetrySync),
                ])
            }

            Event::DataLoaded(Ok(None) | Err(_)) => {
                Command::all([render(), Command::event(Event::ConnectSse)])
            }

            Event::DataSaved(_) => Command::done(),

            // ── Internal: HTTP ──────────────────────────────────────

            Event::ItemsFetched(Ok(mut response)) => {
                if let Some(ref server_items) = response.take_body() {
                    merge_server_items(model, server_items);
                }
                Self::save_state(model)
                    .and(render())
                    .and(Command::event(Event::ConnectSse))
            }

            Event::OpResponse(Ok(mut response)) => {
                if let Some(server_item) = response.take_body() {
                    update_or_insert_item(model, &server_item);
                }
                if !model.pending_ops.is_empty() {
                    model.pending_ops.remove(0);
                }
                model.sync_status = SyncStatus::Idle;
                Self::save_state(model).and(Command::event(Event::RetrySync))
            }

            Event::DeleteOpResponse(Ok(_)) => {
                if !model.pending_ops.is_empty() {
                    model.pending_ops.remove(0);
                }
                model.sync_status = SyncStatus::Idle;
                Self::save_state(model).and(Command::event(Event::RetrySync))
            }

            Event::ItemsFetched(Err(_))
            | Event::OpResponse(Err(_))
            | Event::DeleteOpResponse(Err(_)) => {
                model.sync_status = SyncStatus::Offline;
                render()
            }

            // ── Internal: SSE ───────────────────────────────────────

            Event::SseReceived(msg) => {
                if model.sse_state != SseConnectionState::Connected {
                    model.sse_state = SseConnectionState::Connected;
                }

                match msg.event.as_str() {
                    "item_created" | "item_updated" => {
                        if let Ok(ref server_item) = serde_json::from_str::<TodoItem>(&msg.data) {
                            apply_server_item(model, server_item);
                        }
                    }
                    "item_deleted" => {
                        if let Ok(deleted) =
                            serde_json::from_str::<DeletedItemPayload>(&msg.data)
                        {
                            model.items.retain(|i| i.id != deleted.id);
                            model
                                .pending_ops
                                .retain(|op| op.item_id() != deleted.id);
                        }
                    }
                    _ => {}
                }

                Self::save_state(model).and(render())
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let filtered_items: Vec<TodoItemView> = model
            .items
            .iter()
            .filter(|item| match model.filter {
                Filter::All => true,
                Filter::Active => !item.completed,
                Filter::Completed => item.completed,
            })
            .map(|item| TodoItemView {
                id: item.id.clone(),
                title: item.title.clone(),
                completed: item.completed,
            })
            .collect();

        let active_count = model.items.iter().filter(|i| !i.completed).count();

        ViewModel {
            items: filtered_items,
            input_text: model.input_text.clone(),
            active_count: format!(
                "{active_count} item{} left",
                if active_count == 1 { "" } else { "s" }
            ),
            pending_count: model.pending_ops.len(),
            sync_status: match model.sync_status {
                SyncStatus::Idle => "synced".to_string(),
                SyncStatus::Syncing => "syncing".to_string(),
                SyncStatus::Offline => format!("{} pending", model.pending_ops.len()),
            },
            filter: model.filter.clone(),
        }
    }
}

// ── Conflict resolution helpers ─────────────────────────────────────────

fn update_or_insert_item(model: &mut Model, item: &TodoItem) {
    if let Some(existing) = model.items.iter_mut().find(|i| i.id == item.id) {
        *existing = item.clone();
    } else {
        model.items.push(item.clone());
    }
}

fn merge_server_items(model: &mut Model, server_items: &[TodoItem]) {
    for server_item in server_items {
        apply_server_item(model, server_item);
    }
}

/// Apply a server-sourced item using last-writer-wins conflict resolution.
/// If the server item is at least as recent as any conflicting local mutation,
/// the server version wins and the pending op is removed.
fn apply_server_item(model: &mut Model, server_item: &TodoItem) {
    let has_pending = model
        .pending_ops
        .iter()
        .any(|op| op.item_id() == server_item.id);

    if !has_pending {
        update_or_insert_item(model, server_item);
        return;
    }

    let pending_delete = model.pending_ops.iter().find(|op| {
        matches!(op, PendingOp::Delete { id, .. } if *id == server_item.id)
    });

    if let Some(PendingOp::Delete { deleted_at, .. }) = pending_delete {
        if server_item.updated_at >= *deleted_at {
            model
                .pending_ops
                .retain(|op| op.item_id() != server_item.id);
            update_or_insert_item(model, server_item);
        }
    } else {
        let local_ts = model
            .items
            .iter()
            .find(|i| i.id == server_item.id)
            .map(|i| i.updated_at.as_str());

        let server_wins = local_ts.is_none_or(|ts| server_item.updated_at.as_str() >= ts);

        if server_wins {
            model
                .pending_ops
                .retain(|op| op.item_id() != server_item.id);
            update_or_insert_item(model, server_item);
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crux_kv::KeyValueOperation;

    fn make_item(id: &str, title: &str, completed: bool, ts: &str) -> TodoItem {
        TodoItem {
            id: id.to_string(),
            title: title.to_string(),
            completed,
            updated_at: ts.to_string(),
        }
    }

    fn seeded_model() -> Model {
        Model {
            items: vec![
                make_item("a", "Buy milk", false, "2025-01-01T00:00:00Z"),
                make_item("b", "Walk dog", true, "2025-01-01T00:00:00Z"),
            ],
            ..Model::default()
        }
    }

    // ── Initialize ──────────────────────────────────────────────────

    #[test]
    fn initialize_loads_from_kv() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Initialize, &mut model);

        let request = cmd.expect_one_effect().expect_key_value();
        assert_eq!(
            request.operation,
            KeyValueOperation::Get {
                key: "todo_state".to_string(),
            }
        );
    }

    // ── SetInput ────────────────────────────────────────────────────

    #[test]
    fn set_input_updates_text() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::SetInput("hello".to_string()), &mut model);
        assert_eq!(model.input_text, "hello");
        cmd.expect_one_effect().expect_render();
    }

    // ── AddTodo ─────────────────────────────────────────────────────

    #[test]
    fn add_todo_with_empty_input_is_noop() {
        let app = TodoApp;
        let mut model = Model {
            input_text: "   ".to_string(),
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::AddTodo("id1".to_string(), "2025-06-01T00:00:00Z".to_string()),
            &mut model,
        );
        assert!(cmd.is_done());
        assert!(model.items.is_empty());
    }

    #[test]
    fn add_todo_creates_item_and_queues_op() {
        let app = TodoApp;
        let mut model = Model {
            input_text: "Buy milk".to_string(),
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::AddTodo("id1".to_string(), "2025-06-01T00:00:00Z".to_string()),
            &mut model,
        );

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Buy milk");
        assert_eq!(model.items[0].id, "id1");
        assert!(!model.items[0].completed);
        assert_eq!(model.pending_ops.len(), 1);
        assert!(model.input_text.is_empty());
        assert_eq!(model.sync_status, SyncStatus::Syncing);

        // Effects: KV set (save), render (sync), HTTP POST (sync)
        cmd.expect_effect(); // KV or render
        cmd.expect_effect(); // KV or render
        cmd.expect_one_effect().expect_http();
    }

    // ── EditTitle ───────────────────────────────────────────────────

    #[test]
    fn edit_title_updates_item() {
        let app = TodoApp;
        let mut model = seeded_model();

        let _cmd = app.update(
            Event::EditTitle(
                "a".to_string(),
                "Buy oat milk".to_string(),
                "2025-06-02T00:00:00Z".to_string(),
            ),
            &mut model,
        );

        assert_eq!(model.items[0].title, "Buy oat milk");
        assert_eq!(model.items[0].updated_at, "2025-06-02T00:00:00Z");
        assert_eq!(model.pending_ops.len(), 1);
    }

    #[test]
    fn edit_title_empty_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::EditTitle(
                "a".to_string(),
                "  ".to_string(),
                "2025-06-02T00:00:00Z".to_string(),
            ),
            &mut model,
        );

        assert!(cmd.is_done());
        assert_eq!(model.items[0].title, "Buy milk");
    }

    // ── ToggleCompleted ─────────────────────────────────────────────

    #[test]
    fn toggle_completed_flips_state() {
        let app = TodoApp;
        let mut model = seeded_model();

        assert!(!model.items[0].completed);

        let _cmd = app.update(
            Event::ToggleCompleted("a".to_string(), "2025-06-02T00:00:00Z".to_string()),
            &mut model,
        );

        assert!(model.items[0].completed);
        assert_eq!(model.pending_ops.len(), 1);
    }

    // ── DeleteTodo ──────────────────────────────────────────────────

    #[test]
    fn delete_todo_removes_item_and_queues_op() {
        let app = TodoApp;
        let mut model = seeded_model();

        let _cmd = app.update(
            Event::DeleteTodo("a".to_string(), "2025-06-02T00:00:00Z".to_string()),
            &mut model,
        );

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "b");
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(
            &model.pending_ops[0],
            PendingOp::Delete { id, .. } if id == "a"
        ));
    }

    // ── SetFilter ───────────────────────────────────────────────────

    #[test]
    fn set_filter_changes_filter() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::SetFilter(Filter::Active), &mut model);
        assert_eq!(model.filter, Filter::Active);
        cmd.expect_one_effect().expect_render();
    }

    // ── DataLoaded ──────────────────────────────────────────────────

    #[test]
    fn data_loaded_with_state_restores_items() {
        let app = TodoApp;
        let mut model = Model::default();

        let state = PersistedState {
            items: vec![make_item("x", "Persisted", false, "2025-01-01T00:00:00Z")],
            pending_ops: vec![],
        };
        let bytes = serde_json::to_vec(&state).unwrap();

        let _cmd = app.update(Event::DataLoaded(Ok(Some(bytes))), &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Persisted");
    }

    #[test]
    fn data_loaded_with_none_starts_empty() {
        let app = TodoApp;
        let mut model = Model::default();

        let _cmd = app.update(Event::DataLoaded(Ok(None)), &mut model);
        assert!(model.items.is_empty());
    }

    // ── DataSaved ───────────────────────────────────────────────────

    #[test]
    fn data_saved_is_noop() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::DataSaved(Ok(None)), &mut model);
        assert!(cmd.is_done());
    }

    // ── OpResponse ──────────────────────────────────────────────────

    #[test]
    fn op_response_ok_removes_first_pending_op() {
        let app = TodoApp;
        let item = make_item("a", "Test", false, "2025-01-01T00:00:00Z");
        let mut model = Model {
            pending_ops: vec![PendingOp::Create(item.clone())],
            sync_status: SyncStatus::Syncing,
            ..Model::default()
        };

        let _cmd = app.update(
            Event::OpResponse(Ok(crux_http::testing::ResponseBuilder::ok()
                .body(item)
                .build())),
            &mut model,
        );

        assert!(model.pending_ops.is_empty());
        assert_eq!(model.sync_status, SyncStatus::Idle);
    }

    #[test]
    fn op_response_err_goes_offline() {
        let app = TodoApp;
        let item = make_item("a", "Test", false, "2025-01-01T00:00:00Z");
        let mut model = Model {
            pending_ops: vec![PendingOp::Create(item)],
            sync_status: SyncStatus::Syncing,
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::OpResponse(Err(crux_http::HttpError::Io(
                "network error".to_string(),
            ))),
            &mut model,
        );

        assert_eq!(model.sync_status, SyncStatus::Offline);
        cmd.expect_one_effect().expect_render();
    }

    // ── SseReceived ─────────────────────────────────────────────────

    #[test]
    fn sse_item_created_adds_item() {
        let app = TodoApp;
        let mut model = Model::default();

        let item = make_item("sse1", "From SSE", false, "2025-06-15T10:30:00Z");
        let data = serde_json::to_string(&item).unwrap();

        let _cmd = app.update(
            Event::SseReceived(SseMessage {
                event: "item_created".to_string(),
                data,
            }),
            &mut model,
        );

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "From SSE");
    }

    #[test]
    fn sse_item_deleted_removes_item() {
        let app = TodoApp;
        let mut model = seeded_model();

        let _cmd = app.update(
            Event::SseReceived(SseMessage {
                event: "item_deleted".to_string(),
                data: r#"{"id":"a"}"#.to_string(),
            }),
            &mut model,
        );

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "b");
    }

    // ── SseDisconnected ─────────────────────────────────────────────

    #[test]
    fn sse_disconnected_triggers_refetch() {
        let app = TodoApp;
        let mut model = Model {
            sse_state: SseConnectionState::Connected,
            ..Model::default()
        };

        let mut cmd = app.update(Event::SseDisconnected, &mut model);

        assert_eq!(model.sse_state, SseConnectionState::Disconnected);
        cmd.expect_effect().expect_render();
        let _http = cmd.expect_one_effect().expect_http();
    }

    // ── View ────────────────────────────────────────────────────────

    #[test]
    fn view_filters_active_items() {
        let app = TodoApp;
        let model = Model {
            items: vec![
                make_item("a", "Active", false, "t"),
                make_item("b", "Done", true, "t"),
            ],
            filter: Filter::Active,
            ..Model::default()
        };

        let view = app.view(&model);
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].title, "Active");
    }

    #[test]
    fn view_filters_completed_items() {
        let app = TodoApp;
        let model = Model {
            items: vec![
                make_item("a", "Active", false, "t"),
                make_item("b", "Done", true, "t"),
            ],
            filter: Filter::Completed,
            ..Model::default()
        };

        let view = app.view(&model);
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].title, "Done");
    }

    #[test]
    fn view_shows_correct_active_count() {
        let app = TodoApp;
        let model = Model {
            items: vec![
                make_item("a", "A", false, "t"),
                make_item("b", "B", true, "t"),
                make_item("c", "C", false, "t"),
            ],
            ..Model::default()
        };

        let view = app.view(&model);
        assert_eq!(view.active_count, "2 items left");
    }

    #[test]
    fn view_singular_item_count() {
        let app = TodoApp;
        let model = Model {
            items: vec![make_item("a", "A", false, "t")],
            ..Model::default()
        };

        let view = app.view(&model);
        assert_eq!(view.active_count, "1 item left");
    }

    #[test]
    fn view_shows_sync_status() {
        let app = TodoApp;

        let model = Model {
            sync_status: SyncStatus::Idle,
            ..Model::default()
        };
        assert_eq!(app.view(&model).sync_status, "synced");

        let model = Model {
            sync_status: SyncStatus::Syncing,
            ..Model::default()
        };
        assert_eq!(app.view(&model).sync_status, "syncing");

        let model = Model {
            sync_status: SyncStatus::Offline,
            pending_ops: vec![PendingOp::Create(make_item("x", "X", false, "t"))],
            ..Model::default()
        };
        assert_eq!(app.view(&model).sync_status, "1 pending");
    }

    // ── Conflict resolution ─────────────────────────────────────────

    #[test]
    fn server_item_wins_with_newer_timestamp() {
        let mut model = Model {
            items: vec![make_item("a", "Local", false, "2025-01-01T00:00:00Z")],
            pending_ops: vec![PendingOp::Update(make_item(
                "a",
                "Local",
                false,
                "2025-01-01T00:00:00Z",
            ))],
            ..Model::default()
        };

        let server = make_item("a", "Server", true, "2025-01-02T00:00:00Z");
        apply_server_item(&mut model, &server);

        assert_eq!(model.items[0].title, "Server");
        assert!(model.items[0].completed);
        assert!(model.pending_ops.is_empty());
    }

    #[test]
    fn local_item_wins_with_newer_timestamp() {
        let mut model = Model {
            items: vec![make_item("a", "Local", false, "2025-01-02T00:00:00Z")],
            pending_ops: vec![PendingOp::Update(make_item(
                "a",
                "Local",
                false,
                "2025-01-02T00:00:00Z",
            ))],
            ..Model::default()
        };

        let server = make_item("a", "Server", true, "2025-01-01T00:00:00Z");
        apply_server_item(&mut model, &server);

        assert_eq!(model.items[0].title, "Local");
        assert!(!model.items[0].completed);
        assert_eq!(model.pending_ops.len(), 1);
    }

    #[test]
    fn server_wins_on_timestamp_tie() {
        let mut model = Model {
            items: vec![make_item("a", "Local", false, "2025-01-01T00:00:00Z")],
            pending_ops: vec![PendingOp::Update(make_item(
                "a",
                "Local",
                false,
                "2025-01-01T00:00:00Z",
            ))],
            ..Model::default()
        };

        let server = make_item("a", "Server", true, "2025-01-01T00:00:00Z");
        apply_server_item(&mut model, &server);

        assert_eq!(model.items[0].title, "Server");
        assert!(model.pending_ops.is_empty());
    }

    #[test]
    fn no_conflict_item_is_inserted() {
        let mut model = Model::default();

        let server = make_item("new", "New Item", false, "2025-01-01T00:00:00Z");
        apply_server_item(&mut model, &server);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "New Item");
    }
}
