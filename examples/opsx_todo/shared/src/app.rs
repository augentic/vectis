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
const STORAGE_KEY: &str = "todo_state";

type Http = crux_http::Http<Effect, Event>;
type KeyValue = crux_kv::KeyValue<Effect, Event>;

// ── Domain types ──

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PendingOp {
    Create(TodoItem),
    Update(TodoItem),
    Delete(String),
}

impl PendingOp {
    fn item_id(&self) -> &str {
        match self {
            Self::Create(item) | Self::Update(item) => &item.id,
            Self::Delete(id) => id,
        }
    }
}

#[derive(Serialize)]
struct TodoCreateBody {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Serialize)]
struct TodoUpdateBody {
    title: String,
    completed: bool,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SyncStatus {
    #[default]
    Idle,
    Syncing,
    Offline,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    next_local_id: u64,
}

/// Payload for SSE `item_deleted` events.
#[derive(Deserialize)]
struct DeletePayload {
    id: String,
}

// ── Page (internal) ──

#[derive(Default)]
enum Page {
    #[default]
    Loading,
    TodoList,
    Error,
}

// ── Route (shell-navigable) ──

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum Route {
    #[default]
    TodoList,
}

// ── Model ──

#[derive(Default)]
pub struct Model {
    page: Page,
    items: Vec<TodoItem>,
    pending_ops: Vec<PendingOp>,
    /// Item ID of the currently in-flight sync operation.
    syncing_id: Option<String>,
    filter: Filter,
    sync_status: SyncStatus,
    sse_state: SseConnectionState,
    input_text: String,
    error_message: Option<String>,
    next_local_id: u64,
}

// ── Per-page view structs ──

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct TodoItemView {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct TodoListView {
    pub items: Vec<TodoItemView>,
    pub input_text: String,
    pub active_count: String,
    pub pending_count: String,
    pub sync_status: String,
    pub filter: Filter,
    pub show_clear_completed: bool,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct ErrorView {
    pub message: String,
    pub can_retry: bool,
}

// ── ViewModel ──

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
#[repr(C)]
pub enum ViewModel {
    #[default]
    Loading,
    TodoList(TodoListView),
    Error(ErrorView),
}

// ── Event ──

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Event {
    // Shell-facing events
    Initialize,
    Navigate(Route),
    SetInput(String),
    AddTodo,
    EditTitle(String, String),
    ToggleCompleted(String),
    DeleteTodo(String),
    ClearCompleted,
    SetFilter(Filter),
    RetrySync,
    ConnectSse,
    SseDisconnected,

    // Internal events (effect callbacks)
    #[serde(skip)]
    #[facet(skip)]
    DataLoaded(#[facet(opaque)] Result<Option<Vec<u8>>, KeyValueError>),

    #[serde(skip)]
    #[facet(skip)]
    DataSaved(#[facet(opaque)] Result<Option<Vec<u8>>, KeyValueError>),

    #[serde(skip)]
    #[facet(skip)]
    ItemsFetched(#[facet(opaque)] crux_http::Result<crux_http::Response<Vec<TodoItem>>>),

    #[serde(skip)]
    #[facet(skip)]
    OpResponse(#[facet(opaque)] crux_http::Result<crux_http::Response<TodoItem>>),

    #[serde(skip)]
    #[facet(skip)]
    DeleteOpResponse(#[facet(opaque)] crux_http::Result<crux_http::Response<String>>),

    #[serde(skip)]
    #[facet(skip)]
    SseReceived(#[facet(opaque)] SseMessage),
}

// ── Effect ──

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
    KeyValue(KeyValueOperation),
    ServerSentEvents(SseRequest),
}

// ── App ──

#[derive(Default)]
pub struct TodoApp;

// ── Helpers ──

fn save_state(model: &Model) -> Command<Effect, Event> {
    let state = PersistedState {
        items: model.items.clone(),
        pending_ops: model.pending_ops.clone(),
        next_local_id: model.next_local_id,
    };
    let bytes = serde_json::to_vec(&state).expect("serializing PersistedState");
    KeyValue::set(STORAGE_KEY, bytes).then_send(Event::DataSaved)
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
    model.syncing_id = Some(op.item_id().to_string());

    match op {
        PendingOp::Create(item) => {
            let body = TodoCreateBody {
                id: item.id,
                title: item.title,
                completed: item.completed,
            };
            Http::post(format!("{API_URL}/api/todos"))
                .body_json(&body)
                .expect("serializing TodoCreateBody")
                .expect_json()
                .build()
                .then_send(Event::OpResponse)
        }
        PendingOp::Update(item) => {
            let body = TodoUpdateBody {
                title: item.title,
                completed: item.completed,
            };
            Http::put(format!("{API_URL}/api/todos/{}", item.id))
                .body_json(&body)
                .expect("serializing TodoUpdateBody")
                .expect_json()
                .build()
                .then_send(Event::OpResponse)
        }
        PendingOp::Delete(id) => Http::delete(format!("{API_URL}/api/todos/{id}"))
            .expect_string()
            .build()
            .then_send(Event::DeleteOpResponse),
    }
}

fn update_or_insert_item(items: &mut Vec<TodoItem>, item: &TodoItem) {
    if let Some(existing) = items.iter_mut().find(|i| i.id == item.id) {
        *existing = item.clone();
    } else {
        items.push(item.clone());
    }
}

/// Apply a server-delivered item, respecting last-writer-wins conflict
/// resolution when the item has a pending local mutation.
fn apply_server_item(model: &mut Model, server_item: &TodoItem) {
    let has_pending = model
        .pending_ops
        .iter()
        .any(|op| op.item_id() == server_item.id);

    if has_pending {
        let server_wins = model
            .items
            .iter()
            .find(|i| i.id == server_item.id)
            .is_none_or(|local_item| {
                match (&local_item.updated_at, &server_item.updated_at) {
                    (Some(local_ts), Some(server_ts)) => server_ts >= local_ts,
                    _ => true,
                }
            });

        if server_wins {
            update_or_insert_item(&mut model.items, server_item);
            model
                .pending_ops
                .retain(|op| op.item_id() != server_item.id);
        }
    } else {
        update_or_insert_item(&mut model.items, server_item);
    }
}

/// Merge a full item list from the server with local state, preserving
/// items that have pending create operations not yet on the server.
fn merge_server_items(model: &mut Model, server_items: &[TodoItem]) {
    let mut merged = Vec::new();

    for server_item in server_items {
        let has_pending = model
            .pending_ops
            .iter()
            .any(|op| op.item_id() == server_item.id);

        if has_pending {
            let server_wins = model
                .items
                .iter()
                .find(|i| i.id == server_item.id)
                .is_none_or(|local_item| {
                    match (&local_item.updated_at, &server_item.updated_at) {
                        (Some(local_ts), Some(server_ts)) => server_ts >= local_ts,
                        _ => true,
                    }
                });

            if server_wins {
                merged.push(server_item.clone());
                model
                    .pending_ops
                    .retain(|op| op.item_id() != server_item.id);
            } else if let Some(local_item) =
                model.items.iter().find(|i| i.id == server_item.id)
            {
                merged.push(local_item.clone());
            } else {
                merged.push(server_item.clone());
            }
        } else {
            merged.push(server_item.clone());
        }
    }

    for op in &model.pending_ops {
        if let PendingOp::Create(item) = op
            && !merged.iter().any(|i| i.id == item.id)
        {
            merged.push(item.clone());
        }
    }

    model.items = merged;
}

// ── App implementation ──

#[allow(clippy::too_many_lines)]
impl App for TodoApp {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            Event::Initialize => KeyValue::get(STORAGE_KEY).then_send(Event::DataLoaded),

            Event::Navigate(route) => match route {
                Route::TodoList => match model.page {
                    Page::Error => {
                        model.page = Page::Loading;
                        model.error_message = None;
                        Command::event(Event::Initialize)
                    }
                    Page::Loading | Page::TodoList => Command::done(),
                },
            },

            Event::SetInput(text) => {
                model.input_text = text;
                render()
            }

            Event::AddTodo => {
                let title = model.input_text.trim().to_string();
                if title.is_empty() {
                    return Command::done();
                }

                model.next_local_id += 1;
                let item = TodoItem {
                    id: format!("local_{}", model.next_local_id),
                    title,
                    completed: false,
                    updated_at: None,
                };

                model.items.push(item.clone());
                model.pending_ops.push(PendingOp::Create(item));
                model.input_text = String::new();

                let save = save_state(model);
                let sync = start_sync(model);
                render().and(save).and(sync)
            }

            Event::EditTitle(id, new_title) => {
                if let Some(item) = model.items.iter_mut().find(|i| i.id == id) {
                    item.title = new_title;
                    let updated_item = item.clone();

                    model
                        .pending_ops
                        .retain(|op| op.item_id() != updated_item.id);
                    model.pending_ops.push(PendingOp::Update(updated_item));

                    let save = save_state(model);
                    let sync = start_sync(model);
                    render().and(save).and(sync)
                } else {
                    Command::done()
                }
            }

            Event::ToggleCompleted(id) => {
                if let Some(item) = model.items.iter_mut().find(|i| i.id == id) {
                    item.completed = !item.completed;
                    let updated_item = item.clone();

                    model
                        .pending_ops
                        .retain(|op| op.item_id() != updated_item.id);
                    model.pending_ops.push(PendingOp::Update(updated_item));

                    let save = save_state(model);
                    let sync = start_sync(model);
                    render().and(save).and(sync)
                } else {
                    Command::done()
                }
            }

            Event::DeleteTodo(id) => {
                model.items.retain(|i| i.id != id);
                model.pending_ops.retain(|op| op.item_id() != id);
                model.pending_ops.push(PendingOp::Delete(id));

                let save = save_state(model);
                let sync = start_sync(model);
                render().and(save).and(sync)
            }

            Event::ClearCompleted => {
                let completed_ids: Vec<String> = model
                    .items
                    .iter()
                    .filter(|i| i.completed)
                    .map(|i| i.id.clone())
                    .collect();

                if completed_ids.is_empty() {
                    return Command::done();
                }

                model.items.retain(|i| !i.completed);

                for id in &completed_ids {
                    model.pending_ops.retain(|op| op.item_id() != id);
                    model.pending_ops.push(PendingOp::Delete(id.clone()));
                }

                let save = save_state(model);
                let sync = start_sync(model);
                render().and(save).and(sync)
            }

            Event::SetFilter(filter) => {
                model.filter = filter;
                render()
            }

            Event::RetrySync => {
                let sync = start_sync(model);
                render().and(sync)
            }

            Event::ConnectSse => {
                model.sse_state = SseConnectionState::Connecting;
                ServerSentEvents::get_events(format!("{API_URL}/api/todos/events"))
                    .then_send(Event::SseReceived)
            }

            Event::SseDisconnected => {
                model.sse_state = SseConnectionState::Disconnected;
                Http::get(format!("{API_URL}/api/todos"))
                    .expect_json()
                    .build()
                    .then_send(Event::ItemsFetched)
            }

            // ── Internal event handlers ──

            Event::DataLoaded(Ok(Some(bytes))) => {
                let state: PersistedState =
                    serde_json::from_slice(&bytes).unwrap_or_default();
                model.items = state.items;
                model.pending_ops = state.pending_ops;
                model.next_local_id = state.next_local_id;
                model.page = Page::TodoList;

                Command::all([
                    render(),
                    Command::event(Event::ConnectSse),
                    Http::get(format!("{API_URL}/api/todos"))
                        .expect_json()
                        .build()
                        .then_send(Event::ItemsFetched),
                ])
            }

            Event::DataLoaded(Ok(None)) => {
                model.page = Page::TodoList;

                Command::all([
                    render(),
                    Command::event(Event::ConnectSse),
                    Http::get(format!("{API_URL}/api/todos"))
                        .expect_json()
                        .build()
                        .then_send(Event::ItemsFetched),
                ])
            }

            Event::DataLoaded(Err(e)) => {
                model.page = Page::Error;
                model.error_message = Some(format!("Failed to load data: {e}"));
                render()
            }

            Event::DataSaved(Ok(_) | Err(_)) => Command::done(),

            Event::ItemsFetched(Ok(mut response)) => {
                let server_items = response.take_body().unwrap_or_default();
                merge_server_items(model, &server_items);

                let save = save_state(model);
                let sync = start_sync(model);
                let reconnect = if model.sse_state == SseConnectionState::Disconnected {
                    Command::event(Event::ConnectSse)
                } else {
                    Command::done()
                };
                render().and(save).and(sync).and(reconnect)
            }

            Event::ItemsFetched(Err(_)) => {
                model.sync_status = SyncStatus::Offline;
                render()
            }

            Event::OpResponse(Ok(mut response)) => {
                if let Some(server_item) = response.take_body() {
                    update_or_insert_item(&mut model.items, &server_item);
                }
                if let Some(synced_id) = model.syncing_id.take() {
                    model.pending_ops.retain(|op| op.item_id() != synced_id);
                }
                model.sync_status = SyncStatus::Idle;

                let save = save_state(model);
                let sync = start_sync(model);
                render().and(save).and(sync)
            }

            Event::OpResponse(Err(_)) | Event::DeleteOpResponse(Err(_)) => {
                model.syncing_id = None;
                model.sync_status = SyncStatus::Offline;
                render()
            }

            Event::DeleteOpResponse(Ok(_)) => {
                if let Some(synced_id) = model.syncing_id.take() {
                    model.pending_ops.retain(|op| op.item_id() != synced_id);
                }
                model.sync_status = SyncStatus::Idle;

                let save = save_state(model);
                let sync = start_sync(model);
                render().and(save).and(sync)
            }

            Event::SseReceived(msg) => {
                if model.sse_state != SseConnectionState::Connected {
                    model.sse_state = SseConnectionState::Connected;
                }

                match msg.event.as_str() {
                    "item_created" | "item_updated" => {
                        if let Ok(server_item) =
                            serde_json::from_slice::<TodoItem>(&msg.data)
                        {
                            apply_server_item(model, &server_item);
                            let save = save_state(model);
                            return render().and(save);
                        }
                        Command::done()
                    }
                    "item_deleted" => {
                        if let Ok(payload) =
                            serde_json::from_slice::<DeletePayload>(&msg.data)
                        {
                            model.items.retain(|item| item.id != payload.id);
                            model
                                .pending_ops
                                .retain(|op| op.item_id() != payload.id);
                            let save = save_state(model);
                            return render().and(save);
                        }
                        Command::done()
                    }
                    _ => Command::done(),
                }
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        match model.page {
            Page::Loading => ViewModel::Loading,
            Page::TodoList => {
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
                let pending_count = model.pending_ops.len();
                let has_completed = model.items.iter().any(|i| i.completed);

                let sync_status = match model.sync_status {
                    SyncStatus::Idle if pending_count == 0 => "synced".to_string(),
                    SyncStatus::Idle => format!("{pending_count} pending"),
                    SyncStatus::Syncing => format!("syncing ({pending_count} pending)"),
                    SyncStatus::Offline => "offline".to_string(),
                };

                ViewModel::TodoList(TodoListView {
                    items: filtered_items,
                    input_text: model.input_text.clone(),
                    active_count: format!(
                        "{active_count} item{} left",
                        if active_count == 1 { "" } else { "s" }
                    ),
                    pending_count: pending_count.to_string(),
                    sync_status,
                    filter: model.filter.clone(),
                    show_clear_completed: has_completed,
                })
            }
            Page::Error => ViewModel::Error(ErrorView {
                message: model.error_message.clone().unwrap_or_default(),
                can_retry: true,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crux_core::App;

    fn make_item(id: &str, title: &str, completed: bool) -> TodoItem {
        TodoItem {
            id: id.to_string(),
            title: title.to_string(),
            completed,
            updated_at: Some("2025-06-15T10:30:00Z".to_string()),
        }
    }

    fn seeded_model() -> Model {
        Model {
            page: Page::TodoList,
            items: vec![
                make_item("1", "Buy milk", false),
                make_item("2", "Write tests", false),
                make_item("3", "Done task", true),
            ],
            next_local_id: 3,
            ..Model::default()
        }
    }

    #[test]
    fn initial_view_is_loading() {
        let app = TodoApp;
        let model = Model::default();

        assert!(matches!(app.view(&model), ViewModel::Loading));
    }

    #[test]
    fn initialize_requests_kv_load() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Initialize, &mut model);

        let request = cmd.expect_one_effect().expect_key_value();
        assert_eq!(
            request.operation,
            KeyValueOperation::Get {
                key: STORAGE_KEY.to_string(),
            }
        );
    }

    #[test]
    fn data_loaded_with_state_transitions_to_todo_list() {
        let app = TodoApp;
        let mut model = Model::default();

        let state = PersistedState {
            items: vec![make_item("1", "First", false)],
            pending_ops: vec![],
            next_local_id: 1,
        };
        let bytes = serde_json::to_vec(&state).unwrap();

        let mut cmd = app.update(Event::DataLoaded(Ok(Some(bytes))), &mut model);

        assert_eq!(model.items.len(), 1);
        assert!(matches!(model.page, Page::TodoList));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn data_loaded_none_transitions_to_empty_list() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::DataLoaded(Ok(None)), &mut model);

        assert!(model.items.is_empty());
        assert!(matches!(model.page, Page::TodoList));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn data_loaded_error_transitions_to_error_view() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::DataLoaded(Err(KeyValueError::Io {
                message: "corrupt".to_string(),
            })),
            &mut model,
        );

        assert!(matches!(model.page, Page::Error));
        assert!(model.error_message.is_some());

        cmd.expect_one_effect().expect_render();

        let ViewModel::Error(view) = app.view(&model) else {
            panic!("expected Error view");
        };
        assert!(view.can_retry);
        assert!(view.message.contains("corrupt"));
    }

    #[test]
    fn navigate_from_error_reinitializes() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::Error,
            error_message: Some("failed".to_string()),
            ..Model::default()
        };

        let mut cmd = app.update(Event::Navigate(Route::TodoList), &mut model);

        assert!(matches!(model.page, Page::Loading));
        assert!(model.error_message.is_none());

        let event = cmd.expect_one_event();
        assert_eq!(event, Event::Initialize);
    }

    #[test]
    fn navigate_from_loading_is_noop() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Navigate(Route::TodoList), &mut model);

        assert!(matches!(model.page, Page::Loading));
        assert!(cmd.is_done());
    }

    #[test]
    fn set_input_updates_text() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::SetInput("hello".to_string()), &mut model);

        assert_eq!(model.input_text, "hello");
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn add_todo_creates_item_and_queues_op() {
        let app = TodoApp;
        let mut model = seeded_model();
        model.input_text = "New task".to_string();

        let mut cmd = app.update(Event::AddTodo, &mut model);

        assert_eq!(model.items.len(), 4);
        assert_eq!(model.items[3].title, "New task");
        assert!(!model.items[3].completed);
        assert!(model.input_text.is_empty());
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Create(_)));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn add_todo_with_empty_input_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();
        model.input_text = "   ".to_string();

        let mut cmd = app.update(Event::AddTodo, &mut model);

        assert_eq!(model.items.len(), 3);
        assert!(model.pending_ops.is_empty());
        assert!(cmd.is_done());
    }

    #[test]
    fn edit_title_updates_item_and_queues_op() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::EditTitle("1".to_string(), "Buy oat milk".to_string()),
            &mut model,
        );

        assert_eq!(model.items[0].title, "Buy oat milk");
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Update(_)));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn toggle_completed_flips_flag_and_queues_op() {
        let app = TodoApp;
        let mut model = seeded_model();
        assert!(!model.items[0].completed);

        let mut cmd = app.update(Event::ToggleCompleted("1".to_string()), &mut model);

        assert!(model.items[0].completed);
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Update(_)));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn delete_todo_removes_item_and_queues_op() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::DeleteTodo("2".to_string()), &mut model);

        assert_eq!(model.items.len(), 2);
        assert!(!model.items.iter().any(|i| i.id == "2"));
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Delete(_)));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn clear_completed_removes_completed_items() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::ClearCompleted, &mut model);

        assert_eq!(model.items.len(), 2);
        assert!(model.items.iter().all(|i| !i.completed));
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Delete(_)));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn clear_completed_with_no_completed_is_noop() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            items: vec![make_item("1", "Active", false)],
            ..Model::default()
        };

        let mut cmd = app.update(Event::ClearCompleted, &mut model);

        assert_eq!(model.items.len(), 1);
        assert!(cmd.is_done());
    }

    #[test]
    fn set_filter_changes_filter() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::SetFilter(Filter::Active), &mut model);

        assert_eq!(model.filter, Filter::Active);
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn view_filters_active_items() {
        let app = TodoApp;
        let model = Model {
            page: Page::TodoList,
            items: vec![
                make_item("1", "Active", false),
                make_item("2", "Done", true),
            ],
            filter: Filter::Active,
            ..Model::default()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };

        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].title, "Active");
    }

    #[test]
    fn view_filters_completed_items() {
        let app = TodoApp;
        let model = Model {
            page: Page::TodoList,
            items: vec![
                make_item("1", "Active", false),
                make_item("2", "Done", true),
            ],
            filter: Filter::Completed,
            ..Model::default()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };

        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].title, "Done");
    }

    #[test]
    fn view_shows_active_count() {
        let app = TodoApp;
        let model = seeded_model();

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };

        assert_eq!(view.active_count, "2 items left");
    }

    #[test]
    fn view_shows_singular_item_count() {
        let app = TodoApp;
        let model = Model {
            page: Page::TodoList,
            items: vec![make_item("1", "Only one", false)],
            ..Model::default()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };

        assert_eq!(view.active_count, "1 item left");
    }

    #[test]
    fn view_shows_clear_completed_when_completed_exist() {
        let app = TodoApp;
        let model = seeded_model();

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };

        assert!(view.show_clear_completed);
    }

    #[test]
    fn view_hides_clear_completed_when_none_completed() {
        let app = TodoApp;
        let model = Model {
            page: Page::TodoList,
            items: vec![make_item("1", "Active", false)],
            ..Model::default()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };

        assert!(!view.show_clear_completed);
    }

    #[test]
    fn view_sync_status_synced_when_idle_and_empty() {
        let app = TodoApp;
        let model = Model {
            page: Page::TodoList,
            sync_status: SyncStatus::Idle,
            ..Model::default()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };

        assert_eq!(view.sync_status, "synced");
    }

    #[test]
    fn view_sync_status_offline() {
        let app = TodoApp;
        let model = Model {
            page: Page::TodoList,
            sync_status: SyncStatus::Offline,
            ..Model::default()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };

        assert_eq!(view.sync_status, "offline");
    }

    #[test]
    fn retry_sync_starts_syncing_pending_ops() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            pending_ops: vec![PendingOp::Create(make_item("1", "Test", false))],
            sync_status: SyncStatus::Offline,
            ..Model::default()
        };

        let mut cmd = app.update(Event::RetrySync, &mut model);

        assert_eq!(model.sync_status, SyncStatus::Syncing);
        assert!(model.syncing_id.is_some());

        cmd.expect_effect().expect_render();
        let _request = cmd.expect_one_effect().expect_http();
    }

    #[test]
    fn op_response_success_removes_pending_op() {
        let app = TodoApp;
        let server_item = make_item("1", "Test", false);
        let mut model = Model {
            page: Page::TodoList,
            items: vec![TodoItem {
                updated_at: None,
                ..server_item.clone()
            }],
            pending_ops: vec![PendingOp::Create(server_item.clone())],
            syncing_id: Some("1".to_string()),
            sync_status: SyncStatus::Syncing,
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::OpResponse(Ok(crux_http::testing::ResponseBuilder::ok()
                .body(server_item)
                .build())),
            &mut model,
        );

        assert!(model.pending_ops.is_empty());
        assert!(model.syncing_id.is_none());
        assert_eq!(model.sync_status, SyncStatus::Idle);

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn op_response_error_sets_offline() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            pending_ops: vec![PendingOp::Create(make_item("1", "Test", false))],
            syncing_id: Some("1".to_string()),
            sync_status: SyncStatus::Syncing,
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::OpResponse(Err(crux_http::HttpError::Url(
                "network error".to_string(),
            ))),
            &mut model,
        );

        assert!(model.syncing_id.is_none());
        assert_eq!(model.sync_status, SyncStatus::Offline);
        assert_eq!(model.pending_ops.len(), 1);

        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn sse_received_item_created_adds_item() {
        let app = TodoApp;
        let mut model = seeded_model();

        let msg = SseMessage {
            event: "item_created".to_string(),
            data: serde_json::to_vec(&make_item("new_1", "From server", false)).unwrap(),
        };

        let mut cmd = app.update(Event::SseReceived(msg), &mut model);

        assert_eq!(model.items.len(), 4);
        assert!(model.items.iter().any(|i| i.id == "new_1"));
        assert_eq!(model.sse_state, SseConnectionState::Connected);

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn sse_received_item_updated_applies_conflict_resolution() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            items: vec![make_item("1", "Local title", false)],
            pending_ops: vec![PendingOp::Update(make_item("1", "Local title", false))],
            ..Model::default()
        };

        let newer_item = TodoItem {
            id: "1".to_string(),
            title: "Server title".to_string(),
            completed: true,
            updated_at: Some("2025-06-15T12:00:00Z".to_string()),
        };

        let msg = SseMessage {
            event: "item_updated".to_string(),
            data: serde_json::to_vec(&newer_item).unwrap(),
        };

        let mut cmd = app.update(Event::SseReceived(msg), &mut model);

        assert_eq!(model.items[0].title, "Server title");
        assert!(model.pending_ops.is_empty());

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn sse_received_item_deleted_removes_item() {
        let app = TodoApp;
        let mut model = seeded_model();

        let msg = SseMessage {
            event: "item_deleted".to_string(),
            data: br#"{"id":"2"}"#.to_vec(),
        };

        let mut cmd = app.update(Event::SseReceived(msg), &mut model);

        assert_eq!(model.items.len(), 2);
        assert!(!model.items.iter().any(|i| i.id == "2"));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn connect_sse_sets_connecting_state() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::ConnectSse, &mut model);

        assert_eq!(model.sse_state, SseConnectionState::Connecting);

        let request = cmd.expect_one_effect().expect_server_sent_events();
        assert_eq!(
            request.operation,
            SseRequest {
                url: format!("{API_URL}/api/todos/events"),
            }
        );
    }

    #[test]
    fn sse_disconnected_fetches_items() {
        let app = TodoApp;
        let mut model = seeded_model();
        model.sse_state = SseConnectionState::Connected;

        let mut cmd = app.update(Event::SseDisconnected, &mut model);

        assert_eq!(model.sse_state, SseConnectionState::Disconnected);

        let _request = cmd.expect_one_effect().expect_http();
    }

    #[test]
    fn data_saved_ok_is_silent() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::DataSaved(Ok(None)), &mut model);

        assert!(cmd.is_done());
    }

    #[test]
    fn delete_op_response_success_removes_pending() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            pending_ops: vec![PendingOp::Delete("1".to_string())],
            syncing_id: Some("1".to_string()),
            sync_status: SyncStatus::Syncing,
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::DeleteOpResponse(Ok(crux_http::testing::ResponseBuilder::ok()
                .body(String::new())
                .build())),
            &mut model,
        );

        assert!(model.pending_ops.is_empty());
        assert!(model.syncing_id.is_none());
        assert_eq!(model.sync_status, SyncStatus::Idle);

        cmd.expect_effect().expect_render();
    }
}
