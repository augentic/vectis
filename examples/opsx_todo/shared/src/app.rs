use chrono::{DateTime, Utc};
use crux_core::{
    App, Command,
    macros::effect,
    render::{RenderOperation, render},
};
use crux_http::HttpRequest;
use crux_kv::{KeyValueOperation, error::KeyValueError};
use crux_time::TimeRequest;
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::sse::{ServerSentEvents, SseMessage, SseRequest};

// ── Constants ──────────────────────────────────────────────────────────────────

const API_BASE: &str = "https://api.example.com";
const KV_STATE_KEY: &str = "todos:state";
const RETRY_INTERVAL_SECS: u64 = 30;

// ── Domain types ───────────────────────────────────────────────────────────────

/// A single to-do item.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub updated_at: DateTime<Utc>,
}

/// A queued mutation waiting to be sent to the server.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PendingOp {
    Create(TodoItem),
    Update(TodoItem),
    Delete {
        item_id: String,
        deleted_at: DateTime<Utc>,
    },
}

impl PendingOp {
    fn item_id(&self) -> &str {
        match self {
            Self::Create(item) | Self::Update(item) => &item.id,
            Self::Delete { item_id, .. } => item_id,
        }
    }
}

/// Filter for the visible item list.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

/// Current synchronisation status.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum SyncStatus {
    #[default]
    Idle,
    Syncing,
    Offline,
}

/// SSE connection state.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum SseState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

/// Body for `POST /api/todos`.
#[derive(Serialize)]
struct CreateRequest {
    id: String,
    title: String,
    completed: bool,
}

/// Body for `PUT /api/todos/{id}`.
#[derive(Serialize)]
struct UpdateRequest {
    title: String,
    completed: bool,
}

/// Payload in SSE `item_deleted` events.
#[derive(Deserialize)]
struct DeletedPayload {
    id: String,
}

/// State persisted to key-value storage.
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct PersistedState {
    items: Vec<TodoItem>,
    pending_ops: Vec<PendingOp>,
}

// ── Page (internal) ────────────────────────────────────────────────────────────

#[derive(Default)]
enum Page {
    #[default]
    Loading,
    Error,
    TodoList,
}

// ── Route (shell-facing) ───────────────────────────────────────────────────────

/// Shell-navigable destinations.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum Route {
    #[default]
    TodoList,
}

// ── Model ──────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Model {
    page: Page,
    items: Vec<TodoItem>,
    pending_ops: Vec<PendingOp>,
    filter: Filter,
    sync_status: SyncStatus,
    sse_state: SseState,
    new_title: String,
    /// Item ID of the currently in-flight sync operation.
    syncing_id: Option<String>,
    error_message: Option<String>,
}

// ── Per-page view structs ──────────────────────────────────────────────────────

/// View data for the error page.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct ErrorView {
    pub message: String,
    pub can_retry: bool,
}

/// View data for a single to-do item in the list.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct TodoItemView {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

/// View data for the main to-do list page.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct TodoListView {
    pub items: Vec<TodoItemView>,
    pub new_title: String,
    pub active_count: usize,
    pub has_completed: bool,
    pub filter: Filter,
    pub sync_status: SyncStatus,
    pub sse_state: SseState,
    pub pending_count: usize,
}

// ── ViewModel ──────────────────────────────────────────────────────────────────

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
#[repr(C)]
pub enum ViewModel {
    #[default]
    Loading,
    Error(ErrorView),
    TodoList(TodoListView),
}

// ── Event ──────────────────────────────────────────────────────────────────────

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Event {
    // Shell-facing events
    Navigate(Route),
    SetNewTitle(String),
    AddTodo { id: String },
    EditTitle { id: String, title: String },
    ToggleTodo(String),
    DeleteTodo(String),
    ClearCompleted,
    SetFilter(Filter),

    // Internal events
    #[serde(skip)]
    #[facet(skip)]
    Initialize,

    #[serde(skip)]
    #[facet(skip)]
    DataLoaded(#[facet(opaque)] Result<Option<Vec<u8>>, KeyValueError>),

    #[serde(skip)]
    #[facet(skip)]
    StateSaved(#[facet(opaque)] Result<Option<Vec<u8>>, KeyValueError>),

    #[serde(skip)]
    #[facet(skip)]
    ItemsFetched(#[facet(opaque)] crux_http::Result<crux_http::Response<Vec<TodoItem>>>),

    #[serde(skip)]
    #[facet(skip)]
    OpSynced(#[facet(opaque)] crux_http::Result<crux_http::Response<TodoItem>>),

    #[serde(skip)]
    #[facet(skip)]
    DeleteSynced(#[facet(opaque)] crux_http::Result<crux_http::Response<String>>),

    #[serde(skip)]
    #[facet(skip)]
    SseEvent(#[facet(opaque)] SseMessage),

    #[serde(skip)]
    #[facet(skip)]
    ConnectSse,

    #[serde(skip)]
    #[facet(skip)]
    StartSync,

    #[serde(skip)]
    #[facet(skip)]
    SyncTimerFired(#[facet(opaque)] crux_time::TimerOutcome),

    #[serde(skip)]
    #[facet(skip)]
    CreateWithTime(#[facet(opaque)] std::time::SystemTime, String, String),

    #[serde(skip)]
    #[facet(skip)]
    EditWithTime(#[facet(opaque)] std::time::SystemTime, String, String),

    #[serde(skip)]
    #[facet(skip)]
    ToggleWithTime(#[facet(opaque)] std::time::SystemTime, String),

    #[serde(skip)]
    #[facet(skip)]
    DeleteWithTime(#[facet(opaque)] std::time::SystemTime, String),

    #[serde(skip)]
    #[facet(skip)]
    ClearCompletedWithTime(
        #[facet(opaque)] std::time::SystemTime,
        #[facet(opaque)] Vec<String>,
    ),
}

// ── Effect ─────────────────────────────────────────────────────────────────────

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
    KeyValue(KeyValueOperation),
    Time(TimeRequest),
    ServerSentEvents(SseRequest),
}

// ── Type aliases ───────────────────────────────────────────────────────────────

type Http = crux_http::Http<Effect, Event>;
type KeyValue = crux_kv::KeyValue<Effect, Event>;
type Time = crux_time::Time<Effect, Event>;

// ── App ────────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct TodoApp;

impl App for TodoApp {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    #[allow(clippy::too_many_lines)]
    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            // ── Navigation ─────────────────────────────────────────────────
            Event::Navigate(route) => match route {
                Route::TodoList => match model.page {
                    Page::Error | Page::Loading => {
                        model.page = Page::Loading;
                        model.error_message = None;
                        render().and(Command::event(Event::Initialize))
                    }
                    Page::TodoList => Command::done(),
                },
            },

            // ── Initialization ─────────────────────────────────────────────
            Event::Initialize => {
                KeyValue::get(KV_STATE_KEY).then_send(Event::DataLoaded)
            }

            Event::DataLoaded(Ok(Some(bytes))) => {
                if let Ok(state) =
                    serde_json::from_slice::<PersistedState>(&bytes)
                {
                    model.items = state.items;
                    model.pending_ops = state.pending_ops;
                    model.page = Page::TodoList;

                    Command::all([
                        render(),
                        Command::event(Event::ConnectSse),
                        Command::event(Event::StartSync),
                        fetch_items(),
                    ])
                } else {
                    model.page = Page::TodoList;
                    Command::all([
                        render(),
                        Command::event(Event::ConnectSse),
                        fetch_items(),
                    ])
                }
            }

            Event::DataLoaded(Ok(None)) => {
                model.page = Page::TodoList;
                Command::all([
                    render(),
                    Command::event(Event::ConnectSse),
                    fetch_items(),
                ])
            }

            Event::DataLoaded(Err(e)) => {
                model.page = Page::Error;
                model.error_message = Some(format!("Failed to load data: {e}"));
                render()
            }

            // ── User actions ───────────────────────────────────────────────
            Event::SetNewTitle(title) => {
                model.new_title = title;
                render()
            }

            Event::AddTodo { id } => {
                let title = model.new_title.trim().to_string();
                if title.is_empty() {
                    return Command::done();
                }
                model.new_title = String::new();
                render().and(
                    Time::now().then_send(move |t| Event::CreateWithTime(t, id, title)),
                )
            }

            Event::CreateWithTime(system_time, id, title) => {
                let updated_at: DateTime<Utc> = system_time.into();
                let item = TodoItem {
                    id,
                    title,
                    completed: false,
                    updated_at,
                };
                model.items.push(item.clone());
                model.pending_ops.push(PendingOp::Create(item));
                save_state(model).and(Command::event(Event::StartSync))
            }

            Event::EditTitle { id, title } => {
                let title = title.trim().to_string();
                if title.is_empty() {
                    return Command::done();
                }
                if model.items.iter().any(|i| i.id == id) {
                    Time::now().then_send(move |t| Event::EditWithTime(t, id, title))
                } else {
                    Command::done()
                }
            }

            Event::EditWithTime(system_time, id, title) => {
                let updated_at: DateTime<Utc> = system_time.into();
                if let Some(item) = model.items.iter_mut().find(|i| i.id == id) {
                    item.title = title;
                    item.updated_at = updated_at;
                    model
                        .pending_ops
                        .retain(|op| !matches!(op, PendingOp::Update(i) if i.id == id));
                    model.pending_ops.push(PendingOp::Update(item.clone()));
                    save_state(model).and(Command::event(Event::StartSync))
                } else {
                    Command::done()
                }
            }

            Event::ToggleTodo(id) => {
                if model.items.iter().any(|i| i.id == id) {
                    Time::now().then_send(move |t| Event::ToggleWithTime(t, id))
                } else {
                    Command::done()
                }
            }

            Event::ToggleWithTime(system_time, id) => {
                let updated_at: DateTime<Utc> = system_time.into();
                if let Some(item) = model.items.iter_mut().find(|i| i.id == id) {
                    item.completed = !item.completed;
                    item.updated_at = updated_at;
                    model
                        .pending_ops
                        .retain(|op| !matches!(op, PendingOp::Update(i) if i.id == id));
                    model.pending_ops.push(PendingOp::Update(item.clone()));
                    save_state(model).and(Command::event(Event::StartSync))
                } else {
                    Command::done()
                }
            }

            Event::DeleteTodo(id) => {
                let is_create_only = model
                    .pending_ops
                    .iter()
                    .any(|op| matches!(op, PendingOp::Create(item) if item.id == id));
                model.items.retain(|item| item.id != id);
                if is_create_only {
                    model.pending_ops.retain(|op| op.item_id() != id);
                    save_state(model)
                } else {
                    render().and(
                        Time::now()
                            .then_send(move |t| Event::DeleteWithTime(t, id)),
                    )
                }
            }

            Event::DeleteWithTime(system_time, id) => {
                let deleted_at: DateTime<Utc> = system_time.into();
                model.pending_ops.retain(|op| op.item_id() != id);
                model.pending_ops.push(PendingOp::Delete {
                    item_id: id,
                    deleted_at,
                });
                save_state(model).and(Command::event(Event::StartSync))
            }

            Event::ClearCompleted => {
                let completed_ids: Vec<String> = model
                    .items
                    .iter()
                    .filter(|item| item.completed)
                    .map(|item| item.id.clone())
                    .collect();

                if completed_ids.is_empty() {
                    return Command::done();
                }

                model.items.retain(|item| !item.completed);

                let mut create_only_ids = Vec::new();
                let mut needs_delete_ids = Vec::new();
                for id in completed_ids {
                    if model
                        .pending_ops
                        .iter()
                        .any(|op| matches!(op, PendingOp::Create(item) if item.id == id))
                    {
                        create_only_ids.push(id);
                    } else {
                        needs_delete_ids.push(id);
                    }
                }

                for id in &create_only_ids {
                    model.pending_ops.retain(|op| op.item_id() != *id);
                }

                if needs_delete_ids.is_empty() {
                    return save_state(model);
                }

                render().and(Time::now().then_send(move |t| {
                    Event::ClearCompletedWithTime(t, needs_delete_ids)
                }))
            }

            Event::ClearCompletedWithTime(system_time, ids) => {
                let deleted_at: DateTime<Utc> = system_time.into();
                for id in ids {
                    model.pending_ops.retain(|op| op.item_id() != id);
                    model.pending_ops.push(PendingOp::Delete {
                        item_id: id,
                        deleted_at,
                    });
                }
                save_state(model).and(Command::event(Event::StartSync))
            }

            Event::SetFilter(filter) => {
                model.filter = filter;
                render()
            }

            // ── Server data ────────────────────────────────────────────────
            Event::ItemsFetched(Ok(mut response)) => {
                let server_items = response.take_body().unwrap_or_default();
                for server_item in &server_items {
                    apply_server_item(model, server_item);
                }
                save_state(model)
            }

            #[allow(clippy::match_same_arms)]
            Event::ItemsFetched(Err(_)) => {
                model.sync_status = SyncStatus::Offline;
                render()
            }

            // ── Sync queue ─────────────────────────────────────────────────
            Event::StartSync => start_sync(model),

            Event::OpSynced(Ok(mut response)) => {
                if let Some(server_item) = response.take_body() {
                    apply_server_item(model, &server_item);
                }
                if let Some(synced_id) = model.syncing_id.take() {
                    model.pending_ops.retain(|op| op.item_id() != synced_id);
                }
                model.sync_status = SyncStatus::Idle;
                save_state(model).and(Command::event(Event::StartSync))
            }

            Event::OpSynced(Err(_)) | Event::DeleteSynced(Err(_)) => {
                model.syncing_id = None;
                model.sync_status = SyncStatus::Offline;
                save_state(model).and(start_retry_timer())
            }

            Event::DeleteSynced(Ok(_)) => {
                if let Some(synced_id) = model.syncing_id.take() {
                    model.pending_ops.retain(|op| op.item_id() != synced_id);
                }
                model.sync_status = SyncStatus::Idle;
                save_state(model).and(Command::event(Event::StartSync))
            }

            Event::StateSaved(Ok(_)) => Command::done(),
            Event::StateSaved(Err(_)) => {
                model.sync_status = SyncStatus::Offline;
                render()
            }

            // ── SSE ────────────────────────────────────────────────────────
            Event::ConnectSse => {
                model.sse_state = SseState::Connecting;
                let url = format!("{API_BASE}/api/todos/events");
                render().and(
                    ServerSentEvents::get_events(url).then_send(Event::SseEvent),
                )
            }

            Event::SseEvent(msg) => {
                if model.sse_state != SseState::Connected {
                    model.sse_state = SseState::Connected;
                }
                handle_sse_message(model, &msg)
            }

            // ── Timer ──────────────────────────────────────────────────────
            Event::SyncTimerFired(_) => {
                if model.sync_status == SyncStatus::Offline {
                    model.sync_status = SyncStatus::Idle;
                }
                Command::all([
                    Command::event(Event::StartSync),
                    fetch_items(),
                ])
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        match model.page {
            Page::Loading => ViewModel::Loading,
            Page::Error => ViewModel::Error(ErrorView {
                message: model.error_message.clone().unwrap_or_default(),
                can_retry: true,
            }),
            Page::TodoList => {
                let filtered = filtered_items(&model.items, &model.filter);
                let active_count = model.items.iter().filter(|i| !i.completed).count();
                let has_completed = model.items.iter().any(|i| i.completed);

                ViewModel::TodoList(TodoListView {
                    items: filtered,
                    new_title: model.new_title.clone(),
                    active_count,
                    has_completed,
                    filter: model.filter.clone(),
                    sync_status: model.sync_status.clone(),
                    sse_state: model.sse_state.clone(),
                    pending_count: model.pending_ops.len(),
                })
            }
        }
    }
}

// ── Helper functions ───────────────────────────────────────────────────────────

fn save_state(model: &Model) -> Command<Effect, Event> {
    let state = PersistedState {
        items: model.items.clone(),
        pending_ops: model.pending_ops.clone(),
    };
    let bytes = serde_json::to_vec(&state).expect("serializing PersistedState");
    render().and(KeyValue::set(KV_STATE_KEY, bytes).then_send(Event::StateSaved))
}

fn start_sync(model: &mut Model) -> Command<Effect, Event> {
    if model.pending_ops.is_empty() {
        if model.sync_status != SyncStatus::Idle {
            model.sync_status = SyncStatus::Idle;
            return render();
        }
        return Command::done();
    }
    if model.sync_status == SyncStatus::Syncing {
        return Command::done();
    }

    model.sync_status = SyncStatus::Syncing;
    let op = model.pending_ops[0].clone();
    model.syncing_id = Some(op.item_id().to_string());

    match op {
        PendingOp::Create(item) => {
            let url = format!("{API_BASE}/api/todos");
            let body = CreateRequest {
                id: item.id,
                title: item.title,
                completed: item.completed,
            };
            Http::post(&url)
                .body_json(&body)
                .expect("serializing CreateRequest")
                .expect_json()
                .build()
                .then_send(Event::OpSynced)
        }
        PendingOp::Update(item) => {
            let url = format!("{API_BASE}/api/todos/{}", item.id);
            let body = UpdateRequest {
                title: item.title,
                completed: item.completed,
            };
            Http::put(&url)
                .body_json(&body)
                .expect("serializing UpdateRequest")
                .expect_json()
                .build()
                .then_send(Event::OpSynced)
        }
        PendingOp::Delete { item_id, .. } => {
            let url = format!("{API_BASE}/api/todos/{item_id}");
            Http::delete(&url)
                .expect_string()
                .build()
                .then_send(Event::DeleteSynced)
        }
    }
}

fn start_retry_timer() -> Command<Effect, Event> {
    Time::notify_after(std::time::Duration::from_secs(RETRY_INTERVAL_SECS))
        .0
        .then_send(Event::SyncTimerFired)
}

fn fetch_items() -> Command<Effect, Event> {
    let url = format!("{API_BASE}/api/todos");
    Http::get(&url)
        .expect_json()
        .build()
        .then_send(Event::ItemsFetched)
}

/// Merge a server-provided item using last-writer-wins. The server version wins
/// on timestamp ties. Handles delete conflicts by comparing `deleted_at`.
fn apply_server_item(model: &mut Model, server_item: &TodoItem) {
    let pending_delete_at = model.pending_ops.iter().find_map(|op| {
        if let PendingOp::Delete {
            item_id,
            deleted_at,
        } = op
            && item_id == &server_item.id
        {
            return Some(*deleted_at);
        }
        None
    });

    if let Some(deleted_at) = pending_delete_at {
        if server_item.updated_at >= deleted_at {
            model
                .pending_ops
                .retain(|op| op.item_id() != server_item.id);
            if let Some(local_item) =
                model.items.iter_mut().find(|i| i.id == server_item.id)
            {
                *local_item = server_item.clone();
            } else {
                model.items.push(server_item.clone());
            }
        }
        return;
    }

    if let Some(local_item) = model.items.iter_mut().find(|i| i.id == server_item.id) {
        if server_item.updated_at >= local_item.updated_at {
            *local_item = server_item.clone();
            model
                .pending_ops
                .retain(|op| op.item_id() != server_item.id);
        }
    } else {
        model.items.push(server_item.clone());
    }
}

fn handle_sse_message(model: &mut Model, msg: &SseMessage) -> Command<Effect, Event> {
    match msg.event_type.as_str() {
        "item_created" | "item_updated" => {
            serde_json::from_slice::<TodoItem>(&msg.data).map_or_else(
                |_| Command::done(),
                |server_item| {
                    apply_server_item(model, &server_item);
                    save_state(model)
                },
            )
        }
        "item_deleted" => {
            match serde_json::from_slice::<DeletedPayload>(&msg.data) {
                Ok(payload) => {
                    model.items.retain(|item| item.id != payload.id);
                    let is_syncing =
                        model.syncing_id.as_deref() == Some(payload.id.as_str());
                    if !is_syncing {
                        model
                            .pending_ops
                            .retain(|op| op.item_id() != payload.id);
                    }
                    save_state(model)
                }
                Err(_) => Command::done(),
            }
        }
        _ => Command::done(),
    }
}

fn filtered_items(items: &[TodoItem], filter: &Filter) -> Vec<TodoItemView> {
    items
        .iter()
        .filter(|item| match filter {
            Filter::All => true,
            Filter::Active => !item.completed,
            Filter::Completed => item.completed,
        })
        .map(|item| TodoItemView {
            id: item.id.clone(),
            title: item.title.clone(),
            completed: item.completed,
        })
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crux_time::{Instant as TimeInstant, TimeResponse};

    fn make_item(id: &str, title: &str, completed: bool) -> TodoItem {
        TodoItem {
            id: id.to_string(),
            title: title.to_string(),
            completed,
            updated_at: DateTime::default(),
        }
    }

    fn make_item_at(id: &str, title: &str, completed: bool, secs: i64) -> TodoItem {
        TodoItem {
            id: id.to_string(),
            title: title.to_string(),
            completed,
            updated_at: DateTime::from_timestamp(secs, 0).unwrap(),
        }
    }

    fn seeded_model() -> Model {
        Model {
            page: Page::TodoList,
            items: vec![
                make_item("1", "Buy milk", false),
                make_item("2", "Walk dog", true),
                make_item("3", "Write code", false),
            ],
            ..Model::default()
        }
    }

    /// Consume effects until a `Time` effect is found, resolve it with
    /// `TimeRequest::Now`, and dispatch the resulting event through `update`.
    fn resolve_time(
        app: &TodoApp,
        model: &mut Model,
        cmd: &mut Command<Effect, Event>,
        secs: u64,
    ) -> Command<Effect, Event> {
        loop {
            let effect = cmd.effects().next().expect("expected a Time effect");
            if let Effect::Time(mut request) = effect {
                request
                    .resolve(TimeResponse::Now {
                        instant: TimeInstant::new(secs, 0),
                    })
                    .expect("resolve time");
                let event = cmd.events().next().expect("event after time");
                return app.update(event, model);
            }
        }
    }

    // ── Initialization tests ───────────────────────────────────────────────

    #[test]
    fn initialize_loads_from_kv() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Initialize, &mut model);
        let _request = cmd.expect_one_effect().expect_key_value();
    }

    #[test]
    fn data_loaded_with_items_transitions_to_todo_list() {
        let app = TodoApp;
        let mut model = Model::default();

        let items = vec![make_item("1", "Test", false)];
        let state = PersistedState {
            items,
            pending_ops: vec![],
        };
        let bytes = serde_json::to_vec(&state).unwrap();

        let mut cmd = app.update(Event::DataLoaded(Ok(Some(bytes))), &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Test");
        assert!(matches!(model.page, Page::TodoList));
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn data_loaded_empty_transitions_to_todo_list() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::DataLoaded(Ok(None)), &mut model);

        assert!(model.items.is_empty());
        assert!(matches!(model.page, Page::TodoList));
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn data_loaded_error_shows_error_page() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::DataLoaded(Err(KeyValueError::Io {
                message: "disk full".to_string(),
            })),
            &mut model,
        );

        assert!(matches!(model.page, Page::Error));
        assert!(model.error_message.is_some());
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn corrupted_state_recovers_with_defaults() {
        let app = TodoApp;
        let mut model = Model::default();

        let corrupt_bytes = b"not valid json".to_vec();
        let mut cmd =
            app.update(Event::DataLoaded(Ok(Some(corrupt_bytes))), &mut model);

        assert!(matches!(model.page, Page::TodoList));
        assert!(model.items.is_empty());
        assert!(model.pending_ops.is_empty());
        cmd.expect_effect().expect_render();
    }

    // ── Navigation tests ───────────────────────────────────────────────────

    #[test]
    fn navigate_from_error_reinitializes() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::Error,
            error_message: Some("old error".to_string()),
            ..Model::default()
        };

        let mut cmd = app.update(Event::Navigate(Route::TodoList), &mut model);

        assert!(matches!(model.page, Page::Loading));
        assert!(model.error_message.is_none());
        cmd.expect_effect().expect_render();
        let event = cmd.expect_one_event();
        assert_eq!(event, Event::Initialize);
    }

    // ── AddTodo tests ──────────────────────────────────────────────────────

    #[test]
    fn add_todo_with_valid_title() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            new_title: "Buy groceries".to_string(),
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::AddTodo {
                id: "todo-1".to_string(),
            },
            &mut model,
        );

        assert!(model.new_title.is_empty());
        assert!(model.items.is_empty());

        let _cmd = resolve_time(&app, &mut model, &mut cmd, 1000);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Buy groceries");
        assert!(!model.items[0].completed);
        assert_eq!(model.items[0].id, "todo-1");
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Create(_)));
    }

    #[test]
    fn add_todo_with_empty_title_is_noop() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            new_title: "   ".to_string(),
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::AddTodo {
                id: "todo-1".to_string(),
            },
            &mut model,
        );

        assert!(model.items.is_empty());
        assert!(cmd.is_done());
    }

    // ── Edit / Toggle tests ────────────────────────────────────────────────

    #[test]
    fn edit_title() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::EditTitle {
                id: "1".to_string(),
                title: "Buy oat milk".to_string(),
            },
            &mut model,
        );

        let _cmd = resolve_time(&app, &mut model, &mut cmd, 2000);

        assert_eq!(model.items[0].title, "Buy oat milk");
        assert_eq!(model.pending_ops.len(), 1);
    }

    #[test]
    fn toggle_todo() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd =
            app.update(Event::ToggleTodo("1".to_string()), &mut model);

        let _cmd = resolve_time(&app, &mut model, &mut cmd, 2000);

        assert!(model.items[0].completed);
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Update(_)));
    }

    // ── Delete tests ───────────────────────────────────────────────────────

    #[test]
    fn delete_todo() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd =
            app.update(Event::DeleteTodo("2".to_string()), &mut model);

        assert_eq!(model.items.len(), 2);
        assert!(model.items.iter().all(|i| i.id != "2"));

        let _cmd = resolve_time(&app, &mut model, &mut cmd, 2000);

        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(
            &model.pending_ops[0],
            PendingOp::Delete { item_id, .. } if item_id == "2"
        ));
    }

    #[test]
    fn delete_create_only_eliminates_without_network() {
        let app = TodoApp;
        let item = make_item("new-1", "Unsaved item", false);
        let mut model = Model {
            page: Page::TodoList,
            items: vec![item.clone()],
            pending_ops: vec![PendingOp::Create(item)],
            ..Model::default()
        };

        let _cmd =
            app.update(Event::DeleteTodo("new-1".to_string()), &mut model);

        assert!(model.items.is_empty());
        assert!(model.pending_ops.is_empty());
    }

    // ── ClearCompleted tests ───────────────────────────────────────────────

    #[test]
    fn clear_completed_removes_done_items() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::ClearCompleted, &mut model);

        assert_eq!(model.items.len(), 2);
        assert!(model.items.iter().all(|i| !i.completed));

        let _cmd = resolve_time(&app, &mut model, &mut cmd, 2000);

        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(
            &model.pending_ops[0],
            PendingOp::Delete { item_id, .. } if item_id == "2"
        ));
    }

    #[test]
    fn clear_completed_eliminates_create_only() {
        let app = TodoApp;
        let mut item = make_item("new-1", "Unsaved done", false);
        item.completed = true;
        let mut model = Model {
            page: Page::TodoList,
            items: vec![
                make_item("1", "Keep this", false),
                item.clone(),
            ],
            pending_ops: vec![PendingOp::Create(item)],
            ..Model::default()
        };

        let _cmd = app.update(Event::ClearCompleted, &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "1");
        assert!(model.pending_ops.is_empty());
    }

    #[test]
    fn clear_completed_with_no_completed_is_noop() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            items: vec![
                make_item("1", "Active 1", false),
                make_item("2", "Active 2", false),
            ],
            ..Model::default()
        };

        let mut cmd = app.update(Event::ClearCompleted, &mut model);

        assert_eq!(model.items.len(), 2);
        assert!(cmd.is_done());
    }

    // ── Filter tests ───────────────────────────────────────────────────────

    #[test]
    fn set_filter_updates_view() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::SetFilter(Filter::Active), &mut model);

        assert_eq!(model.filter, Filter::Active);
        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        if let ViewModel::TodoList(list) = view {
            assert_eq!(list.items.len(), 2);
            assert!(list.items.iter().all(|i| !i.completed));
        } else {
            panic!("Expected TodoList view");
        }
    }

    #[test]
    fn set_new_title() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            ..Model::default()
        };

        let mut cmd =
            app.update(Event::SetNewTitle("Hello".to_string()), &mut model);

        assert_eq!(model.new_title, "Hello");
        cmd.expect_one_effect().expect_render();
    }

    // ── SSE tests ──────────────────────────────────────────────────────────

    #[test]
    fn sse_item_created() {
        let app = TodoApp;
        let mut model = seeded_model();
        model.sse_state = SseState::Connected;

        let new_item = make_item("4", "New task", false);
        let data = serde_json::to_vec(&new_item).unwrap();

        let _cmd = app.update(
            Event::SseEvent(SseMessage {
                event_type: "item_created".to_string(),
                data,
            }),
            &mut model,
        );

        assert_eq!(model.items.len(), 4);
        assert_eq!(model.items[3].title, "New task");
    }

    #[test]
    fn sse_item_deleted() {
        let app = TodoApp;
        let mut model = seeded_model();
        model.sse_state = SseState::Connected;

        let data =
            serde_json::to_vec(&serde_json::json!({"id": "2"})).unwrap();

        let _cmd = app.update(
            Event::SseEvent(SseMessage {
                event_type: "item_deleted".to_string(),
                data,
            }),
            &mut model,
        );

        assert_eq!(model.items.len(), 2);
        assert!(model.items.iter().all(|i| i.id != "2"));
    }

    #[test]
    fn sse_deleted_during_sync_preserves_pending_ops() {
        let app = TodoApp;
        let mut model = seeded_model();
        model.sse_state = SseState::Connected;
        model.syncing_id = Some("2".to_string());
        model.pending_ops.push(PendingOp::Update(make_item(
            "2",
            "Walk dog updated",
            true,
        )));

        let data =
            serde_json::to_vec(&serde_json::json!({"id": "2"})).unwrap();
        let _cmd = app.update(
            Event::SseEvent(SseMessage {
                event_type: "item_deleted".to_string(),
                data,
            }),
            &mut model,
        );

        assert_eq!(model.items.len(), 2);
        assert!(model.items.iter().all(|i| i.id != "2"));
        assert_eq!(model.pending_ops.len(), 1);
        assert_eq!(model.pending_ops[0].item_id(), "2");
    }

    // ── Conflict resolution tests ──────────────────────────────────────────

    #[test]
    fn delete_conflict_server_wins() {
        let mut model = Model {
            page: Page::TodoList,
            items: vec![],
            pending_ops: vec![PendingOp::Delete {
                item_id: "1".to_string(),
                deleted_at: DateTime::from_timestamp(1000, 0).unwrap(),
            }],
            ..Model::default()
        };

        let server_item = make_item_at("1", "Server version", false, 2000);
        apply_server_item(&mut model, &server_item);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Server version");
        assert!(model.pending_ops.is_empty());
    }

    #[test]
    fn delete_conflict_local_wins() {
        let mut model = Model {
            page: Page::TodoList,
            items: vec![],
            pending_ops: vec![PendingOp::Delete {
                item_id: "1".to_string(),
                deleted_at: DateTime::from_timestamp(2000, 0).unwrap(),
            }],
            ..Model::default()
        };

        let server_item = make_item_at("1", "Stale server", false, 1000);
        apply_server_item(&mut model, &server_item);

        assert!(model.items.is_empty());
        assert_eq!(model.pending_ops.len(), 1);
    }

    // ── Operation coalescing tests ─────────────────────────────────────────

    #[test]
    fn rapid_toggle_coalesces_updates() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd =
            app.update(Event::ToggleTodo("1".to_string()), &mut model);
        let _cmd = resolve_time(&app, &mut model, &mut cmd, 1000);

        assert_eq!(model.pending_ops.len(), 1);

        let mut cmd =
            app.update(Event::ToggleTodo("1".to_string()), &mut model);
        let _cmd = resolve_time(&app, &mut model, &mut cmd, 2000);

        assert_eq!(model.pending_ops.len(), 1);
        if let PendingOp::Update(item) = &model.pending_ops[0] {
            assert!(!item.completed);
            assert_eq!(
                item.updated_at,
                DateTime::from_timestamp(2000, 0).unwrap()
            );
        } else {
            panic!("Expected Update op");
        }
    }

    // ── Sync tests ─────────────────────────────────────────────────────────

    #[test]
    fn start_sync_with_pending_create() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            pending_ops: vec![PendingOp::Create(make_item("1", "Test", false))],
            ..Model::default()
        };

        let mut cmd = app.update(Event::StartSync, &mut model);

        assert_eq!(model.sync_status, SyncStatus::Syncing);
        assert_eq!(model.syncing_id.as_deref(), Some("1"));
        let _request = cmd.expect_one_effect().expect_http();
    }

    #[test]
    fn start_sync_empty_queue_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::StartSync, &mut model);

        assert!(cmd.is_done());
    }

    // ── SSE connection tests ───────────────────────────────────────────────

    #[test]
    fn connect_sse() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            ..Model::default()
        };

        let mut cmd = app.update(Event::ConnectSse, &mut model);

        assert_eq!(model.sse_state, SseState::Connecting);
        cmd.expect_effect().expect_render();
        let _request = cmd.expect_one_effect().expect_server_sent_events();
    }

    // ── State persistence tests ────────────────────────────────────────────

    #[test]
    fn state_saved_ok_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::StateSaved(Ok(None)), &mut model);

        assert!(cmd.is_done());
    }

    #[test]
    fn state_saved_error_sets_offline() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::StateSaved(Err(KeyValueError::Io {
                message: "write failed".to_string(),
            })),
            &mut model,
        );

        assert_eq!(model.sync_status, SyncStatus::Offline);
        cmd.expect_one_effect().expect_render();
    }

    // ── View tests ─────────────────────────────────────────────────────────

    #[test]
    fn view_loading() {
        let app = TodoApp;
        let model = Model::default();
        let view = app.view(&model);
        assert!(matches!(view, ViewModel::Loading));
    }

    #[test]
    fn view_error() {
        let app = TodoApp;
        let model = Model {
            page: Page::Error,
            error_message: Some("Something went wrong".to_string()),
            ..Model::default()
        };

        let view = app.view(&model);
        match view {
            ViewModel::Error(err) => {
                assert_eq!(err.message, "Something went wrong");
                assert!(err.can_retry);
            }
            _ => panic!("Expected Error view"),
        }
    }

    #[test]
    fn view_todo_list() {
        let app = TodoApp;
        let model = seeded_model();

        let view = app.view(&model);
        match view {
            ViewModel::TodoList(list) => {
                assert_eq!(list.items.len(), 3);
                assert_eq!(list.active_count, 2);
                assert!(list.has_completed);
                assert_eq!(list.filter, Filter::All);
                assert_eq!(list.sync_status, SyncStatus::Idle);
                assert_eq!(list.sse_state, SseState::Disconnected);
            }
            _ => panic!("Expected TodoList view"),
        }
    }
}
