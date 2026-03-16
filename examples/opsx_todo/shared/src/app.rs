#![allow(clippy::cargo_common_metadata)]

use crux_core::{
    macros::effect,
    render::{render, RenderOperation},
    App, Command,
};
use crux_http::HttpRequest;
use crux_kv::{KeyValueOperation, error::KeyValueError};
use crux_time::TimeRequest;
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::sse::{ServerSentEvents, SseMessage, SseRequest};

const API_URL: &str = "https://api.example.com/api/todos";
const STORAGE_KEY_ITEMS: &str = "todo_items";
const STORAGE_KEY_OPS: &str = "todo_pending_ops";
const RETRY_DELAY_SECS: u64 = 30;

type Http = crux_http::Http<Effect, Event>;
type KeyValue = crux_kv::KeyValue<Effect, Event>;
type Time = crux_time::Time<Effect, Event>;

// ── Domain types ──────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub updated_at: String,
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

/// Payload shape for the `item_deleted` SSE event.
#[derive(Deserialize)]
struct DeletePayload {
    id: String,
}

// ── Page (internal) ───────────────────────────────────────────

#[derive(Default)]
enum Page {
    #[default]
    Loading,
    Error,
    TodoList,
}

// ── Route (shell-navigable) ──────────────────────────────────

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum Route {
    #[default]
    TodoList,
}

// ── Model ─────────────────────────────────────────────────────

#[derive(Default)]
pub struct Model {
    page: Page,
    items: Vec<TodoItem>,
    pending_ops: Vec<PendingOp>,
    /// Item ID of the currently in-flight sync operation.
    syncing_id: Option<String>,
    filter: Filter,
    sync_status: SyncStatus,
    last_synced_at: Option<String>,
    sse_state: SseConnectionState,
    error_message: Option<String>,
}

// ── Per-page view structs ─────────────────────────────────────

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct TodoItemView {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct TodoListView {
    pub items: Vec<TodoItemView>,
    pub active_count: String,
    pub has_completed: bool,
    pub filter: Filter,
    pub sync_status: String,
    pub sync_detail: String,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
pub struct ErrorView {
    pub message: String,
    pub can_retry: bool,
}

// ── ViewModel ─────────────────────────────────────────────────

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]
#[repr(C)]
pub enum ViewModel {
    #[default]
    Loading,
    TodoList(TodoListView),
    Error(ErrorView),
}

// ── Events ────────────────────────────────────────────────────

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Event {
    // Shell-facing events
    Navigate(Route),
    /// `(id, title)` -- shell generates the CUID v2 id.
    AddItem(String, String),
    /// `(id, new_title)`
    EditTitle(String, String),
    ToggleCompleted(String),
    DeleteItem(String),
    ClearCompleted,
    SetFilter(Filter),

    // Internal events
    #[serde(skip)]
    #[facet(skip)]
    Loaded(#[facet(opaque)] Result<Option<Vec<u8>>, KeyValueError>),

    #[serde(skip)]
    #[facet(skip)]
    Saved(#[facet(opaque)] Result<Option<Vec<u8>>, KeyValueError>),

    #[serde(skip)]
    #[facet(skip)]
    ItemsFetched(
        #[facet(opaque)] crux_http::Result<crux_http::Response<Vec<TodoItem>>>,
    ),

    #[serde(skip)]
    #[facet(skip)]
    OpSynced(#[facet(opaque)] crux_http::Result<crux_http::Response<TodoItem>>),

    #[serde(skip)]
    #[facet(skip)]
    DeleteSynced(#[facet(opaque)] crux_http::Result<crux_http::Response<String>>),

    #[serde(skip)]
    #[facet(skip)]
    SseEvent(SseMessage),

    #[serde(skip)]
    #[facet(skip)]
    RetrySync,

    #[serde(skip)]
    #[facet(skip)]
    ConnectSse,

    #[serde(skip)]
    #[facet(skip)]
    StartRetryTimer,

    #[serde(skip)]
    #[facet(skip)]
    RetryTimerFired(#[facet(opaque)] crux_time::TimerOutcome),
}

// ── Effects ───────────────────────────────────────────────────

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
    KeyValue(KeyValueOperation),
    Time(TimeRequest),
    ServerSentEvents(SseRequest),
}

// ── App ───────────────────────────────────────────────────────

#[derive(Default)]
pub struct TodoApp;

impl TodoApp {
    fn save_state(model: &Model) -> Command<Effect, Event> {
        let items_bytes = serde_json::to_vec(&model.items).unwrap_or_default();
        let ops_bytes =
            serde_json::to_vec(&model.pending_ops).unwrap_or_default();

        KeyValue::set(STORAGE_KEY_ITEMS, items_bytes)
            .then_send(Event::Saved)
            .and(
                KeyValue::set(STORAGE_KEY_OPS, ops_bytes)
                    .then_send(Event::Saved),
            )
    }

    fn start_sync(model: &mut Model) -> Command<Effect, Event> {
        if model.pending_ops.is_empty() {
            if model.sync_status == SyncStatus::Syncing {
                model.sync_status = SyncStatus::Idle;
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
            PendingOp::Create(item) => Http::post(API_URL)
                .body_json(&item)
                .expect("serialize create body")
                .expect_json()
                .build()
                .then_send(Event::OpSynced),
            PendingOp::Update(item) => {
                let url = format!("{API_URL}/{}", item.id);
                Http::put(url)
                    .body_json(&item)
                    .expect("serialize update body")
                    .expect_json()
                    .build()
                    .then_send(Event::OpSynced)
            }
            PendingOp::Delete(id) => {
                let url = format!("{API_URL}/{id}");
                Http::delete(url)
                    .expect_string()
                    .build()
                    .then_send(Event::DeleteSynced)
            }
        }
    }

    fn apply_server_item(model: &mut Model, server_item: &TodoItem) {
        if let Some(local) = model.items.iter_mut().find(|i| i.id == server_item.id)
        {
            if server_item.updated_at >= local.updated_at {
                *local = server_item.clone();
            }
        } else {
            model.items.push(server_item.clone());
        }
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
            // ── Navigation ───────────────────────────────────
            Event::Navigate(route) => match route {
                Route::TodoList => match model.page {
                    Page::Error | Page::Loading => {
                        model.page = Page::Loading;
                        model.error_message = None;
                        render().and(
                            KeyValue::get(STORAGE_KEY_ITEMS)
                                .then_send(Event::Loaded),
                        )
                    }
                    Page::TodoList => Command::done(),
                },
            },

            // ── Initialisation (KV load) ─────────────────────
            Event::Loaded(Ok(Some(bytes))) => {
                let items: Vec<TodoItem> =
                    serde_json::from_slice(&bytes).unwrap_or_default();
                model.items = items;
                model.page = Page::TodoList;

                render()
                    .and(Command::event(Event::ConnectSse))
                    .and(Self::start_sync(model))
            }

            Event::Loaded(Ok(None)) => {
                model.items = Vec::new();
                model.page = Page::TodoList;

                render()
                    .and(Command::event(Event::ConnectSse))
                    .and(
                        Http::get(API_URL)
                            .expect_json()
                            .build()
                            .then_send(Event::ItemsFetched),
                    )
            }

            Event::Loaded(Err(e)) => {
                model.page = Page::Error;
                model.error_message =
                    Some(format!("Failed to load data: {e}"));
                render()
            }

            // ── User actions ─────────────────────────────────
            Event::AddItem(id, title) => {
                let trimmed = title.trim();
                if trimmed.is_empty() {
                    return Command::done();
                }

                let now = chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string();
                let item = TodoItem {
                    id,
                    title: trimmed.to_string(),
                    completed: false,
                    updated_at: now,
                };

                model.items.push(item.clone());
                model
                    .pending_ops
                    .push(PendingOp::Create(item));

                render()
                    .and(Self::save_state(model))
                    .and(Self::start_sync(model))
            }

            Event::EditTitle(id, new_title) => {
                let trimmed = new_title.trim();
                if trimmed.is_empty() {
                    return Command::done();
                }

                let Some(item) =
                    model.items.iter_mut().find(|i| i.id == id)
                else {
                    return Command::done();
                };
                item.title = trimmed.to_string();
                item.updated_at = chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string();
                let updated = item.clone();
                model.pending_ops.push(PendingOp::Update(updated));

                render()
                    .and(Self::save_state(model))
                    .and(Self::start_sync(model))
            }

            Event::ToggleCompleted(id) => {
                let Some(item) =
                    model.items.iter_mut().find(|i| i.id == id)
                else {
                    return Command::done();
                };
                item.completed = !item.completed;
                item.updated_at = chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string();
                let updated = item.clone();
                model.pending_ops.push(PendingOp::Update(updated));

                render()
                    .and(Self::save_state(model))
                    .and(Self::start_sync(model))
            }

            Event::DeleteItem(id) => {
                let len_before = model.items.len();
                model.items.retain(|i| i.id != id);
                if model.items.len() < len_before {
                    model.pending_ops.push(PendingOp::Delete(id));
                }

                render()
                    .and(Self::save_state(model))
                    .and(Self::start_sync(model))
            }

            Event::ClearCompleted => {
                let completed_ids: Vec<String> = model
                    .items
                    .iter()
                    .filter(|i| i.completed)
                    .map(|i| i.id.clone())
                    .collect();

                model.items.retain(|i| !i.completed);
                for id in completed_ids {
                    model.pending_ops.push(PendingOp::Delete(id));
                }

                render()
                    .and(Self::save_state(model))
                    .and(Self::start_sync(model))
            }

            Event::SetFilter(filter) => {
                model.filter = filter;
                render()
            }

            // ── KV save confirmation ─────────────────────────
            Event::Saved(Ok(_) | Err(_)) => Command::done(),

            // ── HTTP: fetch all items ────────────────────────
            Event::ItemsFetched(Ok(mut response)) => {
                if let Some(items) = response.take_body() {
                    model.items = items;
                    model.last_synced_at = Some(
                        chrono::Utc::now()
                            .format("%Y-%m-%dT%H:%M:%SZ")
                            .to_string(),
                    );
                }
                render().and(Self::save_state(model))
            }

            Event::ItemsFetched(Err(_)) => {
                model.sync_status = SyncStatus::Offline;
                render()
            }

            // ── Sync: create/update response ─────────────────
            Event::OpSynced(Ok(mut response)) => {
                if let Some(server_item) = response.take_body() {
                    Self::apply_server_item(model, &server_item);
                }
                if let Some(synced_id) = model.syncing_id.take() {
                    model
                        .pending_ops
                        .retain(|op| op.item_id() != synced_id);
                }
                model.sync_status = SyncStatus::Idle;
                model.last_synced_at = Some(
                    chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%SZ")
                        .to_string(),
                );

                render()
                    .and(Self::save_state(model))
                    .and(Command::event(Event::RetrySync))
            }

            // ── Sync: delete response ────────────────────────
            Event::DeleteSynced(Ok(_)) => {
                if let Some(synced_id) = model.syncing_id.take() {
                    model
                        .pending_ops
                        .retain(|op| op.item_id() != synced_id);
                }
                model.sync_status = SyncStatus::Idle;
                model.last_synced_at = Some(
                    chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%SZ")
                        .to_string(),
                );

                render()
                    .and(Self::save_state(model))
                    .and(Command::event(Event::RetrySync))
            }

            Event::OpSynced(Err(_)) | Event::DeleteSynced(Err(_)) => {
                model.syncing_id = None;
                model.sync_status = SyncStatus::Offline;
                render().and(Command::event(Event::StartRetryTimer))
            }

            // ── Sync retry ───────────────────────────────────
            Event::RetrySync => Self::start_sync(model),

            Event::StartRetryTimer => {
                let (cmd, _handle) = Time::notify_after(
                    std::time::Duration::from_secs(RETRY_DELAY_SECS),
                );
                cmd.then_send(Event::RetryTimerFired)
            }

            Event::RetryTimerFired(_) => {
                model.sync_status = SyncStatus::Idle;
                render().and(Self::start_sync(model))
            }

            // ── SSE ──────────────────────────────────────────
            Event::ConnectSse => {
                model.sse_state = SseConnectionState::Connecting;
                let url = format!("{API_URL}/events");
                ServerSentEvents::get::<Effect, Event>(&url)
                    .then_send(Event::SseEvent)
            }

            Event::SseEvent(msg) => {
                if model.sse_state != SseConnectionState::Connected {
                    model.sse_state = SseConnectionState::Connected;
                }

                match msg.event_type.as_str() {
                    "item_created" | "item_updated" => {
                        if let Ok(server_item) =
                            serde_json::from_str::<TodoItem>(&msg.data)
                        {
                            Self::apply_server_item(model, &server_item);
                        }
                    }
                    "item_deleted" => {
                        if let Ok(payload) =
                            serde_json::from_str::<DeletePayload>(&msg.data)
                        {
                            model.items.retain(|i| i.id != payload.id);
                            model
                                .pending_ops
                                .retain(|op| op.item_id() != payload.id);
                        }
                    }
                    _ => {}
                }

                render().and(Self::save_state(model))
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

                let active_count =
                    model.items.iter().filter(|i| !i.completed).count();
                let has_completed = model.items.iter().any(|i| i.completed);

                let (sync_status, sync_detail) = match &model.sync_status {
                    SyncStatus::Idle => {
                        if model.pending_ops.is_empty() {
                            (
                                "synced".to_string(),
                                model
                                    .last_synced_at
                                    .as_ref()
                                    .map_or_else(String::new, |t| {
                                        format!("Last synced: {t}")
                                    }),
                            )
                        } else {
                            (
                                "pending".to_string(),
                                format!(
                                    "{} change{} pending",
                                    model.pending_ops.len(),
                                    if model.pending_ops.len() == 1 {
                                        ""
                                    } else {
                                        "s"
                                    }
                                ),
                            )
                        }
                    }
                    SyncStatus::Syncing => (
                        "syncing".to_string(),
                        "Syncing changes…".to_string(),
                    ),
                    SyncStatus::Offline => (
                        "offline".to_string(),
                        format!(
                            "{} change{} queued",
                            model.pending_ops.len(),
                            if model.pending_ops.len() == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ),
                    ),
                };

                ViewModel::TodoList(TodoListView {
                    items: filtered_items,
                    active_count: format!(
                        "{active_count} item{} left",
                        if active_count == 1 { "" } else { "s" }
                    ),
                    has_completed,
                    filter: model.filter.clone(),
                    sync_status,
                    sync_detail,
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
            updated_at: "2025-06-15T10:00:00Z".to_string(),
        }
    }

    fn seeded_model() -> Model {
        Model {
            page: Page::TodoList,
            items: vec![
                make_item("1", "Buy milk", false),
                make_item("2", "Walk dog", true),
            ],
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
    fn navigate_from_error_reloads() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::Error,
            error_message: Some("failed".to_string()),
            ..Model::default()
        };

        let mut cmd =
            app.update(Event::Navigate(Route::TodoList), &mut model);

        assert!(matches!(model.page, Page::Loading));
        assert!(model.error_message.is_none());
        cmd.expect_effect().expect_render();
        let _request = cmd.expect_one_effect().expect_key_value();
    }

    #[test]
    fn navigate_from_loading_triggers_load() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Navigate(Route::TodoList), &mut model);
        assert!(matches!(model.page, Page::Loading));
        cmd.expect_effect().expect_render();
        let _request = cmd.expect_one_effect().expect_key_value();
    }

    #[test]
    fn loaded_with_data_transitions_to_todo_list() {
        let app = TodoApp;
        let mut model = Model::default();

        let items = vec![make_item("1", "Buy milk", false)];
        let bytes = serde_json::to_vec(&items).unwrap();

        let mut cmd = app.update(Event::Loaded(Ok(Some(bytes))), &mut model);

        assert_eq!(model.items.len(), 1);
        assert!(matches!(model.page, Page::TodoList));
        // Should produce render + other effects
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn loaded_with_none_transitions_to_empty_list() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Loaded(Ok(None)), &mut model);

        assert!(model.items.is_empty());
        assert!(matches!(model.page, Page::TodoList));
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn loaded_error_transitions_to_error_view() {
        let app = TodoApp;
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::Loaded(Err(KeyValueError::Io {
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
    }

    #[test]
    fn add_item_with_blank_title_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();
        let count_before = model.items.len();

        let mut cmd = app.update(
            Event::AddItem("3".to_string(), "   ".to_string()),
            &mut model,
        );

        assert_eq!(model.items.len(), count_before);
        assert!(cmd.is_done());
    }

    #[test]
    fn add_item_appends_and_queues_create() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::AddItem("3".to_string(), "New task".to_string()),
            &mut model,
        );

        assert_eq!(model.items.len(), 3);
        assert_eq!(model.items[2].title, "New task");
        assert!(!model.items[2].completed);
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Create(item) if item.id == "3"));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn edit_title_updates_item_and_queues_update() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::EditTitle("1".to_string(), "Buy oat milk".to_string()),
            &mut model,
        );

        assert_eq!(model.items[0].title, "Buy oat milk");
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Update(item) if item.title == "Buy oat milk"));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn toggle_completed_flips_flag_and_queues_update() {
        let app = TodoApp;
        let mut model = seeded_model();
        assert!(!model.items[0].completed);

        let mut cmd = app.update(
            Event::ToggleCompleted("1".to_string()),
            &mut model,
        );

        assert!(model.items[0].completed);
        assert_eq!(model.pending_ops.len(), 1);

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn delete_item_removes_and_queues_delete() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd =
            app.update(Event::DeleteItem("1".to_string()), &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "2");
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Delete(id) if id == "1"));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn clear_completed_removes_all_completed() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::ClearCompleted, &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "1");
        assert_eq!(model.pending_ops.len(), 1);
        assert!(matches!(&model.pending_ops[0], PendingOp::Delete(id) if id == "2"));

        cmd.expect_effect().expect_render();
    }

    #[test]
    fn set_filter_changes_filter() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd =
            app.update(Event::SetFilter(Filter::Active), &mut model);

        assert_eq!(model.filter, Filter::Active);
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn view_filters_active_items() {
        let app = TodoApp;
        let model = Model {
            filter: Filter::Active,
            ..seeded_model()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].id, "1");
    }

    #[test]
    fn view_filters_completed_items() {
        let app = TodoApp;
        let model = Model {
            filter: Filter::Completed,
            ..seeded_model()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].id, "2");
    }

    #[test]
    fn view_shows_all_items_by_default() {
        let app = TodoApp;
        let model = seeded_model();

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };
        assert_eq!(view.items.len(), 2);
        assert_eq!(view.active_count, "1 item left");
        assert!(view.has_completed);
    }

    #[test]
    fn view_pluralizes_item_count() {
        let app = TodoApp;
        let model = Model {
            page: Page::TodoList,
            items: vec![
                make_item("1", "A", false),
                make_item("2", "B", false),
            ],
            ..Model::default()
        };

        let ViewModel::TodoList(view) = app.view(&model) else {
            panic!("expected TodoList view");
        };
        assert_eq!(view.active_count, "2 items left");
        assert!(!view.has_completed);
    }

    #[test]
    fn retry_sync_starts_sync_when_ops_pending() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            pending_ops: vec![PendingOp::Create(make_item("1", "Test", false))],
            ..Model::default()
        };

        let mut cmd = app.update(Event::RetrySync, &mut model);

        assert_eq!(model.sync_status, SyncStatus::Syncing);
        assert_eq!(model.syncing_id.as_deref(), Some("1"));
        let _request = cmd.expect_one_effect().expect_http();
    }

    #[test]
    fn retry_sync_noop_when_empty_queue() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            ..Model::default()
        };

        let mut cmd = app.update(Event::RetrySync, &mut model);
        assert!(cmd.is_done());
    }

    #[test]
    fn sse_event_upserts_item() {
        let app = TodoApp;
        let mut model = seeded_model();

        let msg = SseMessage {
            event_type: "item_updated".to_string(),
            data: serde_json::to_string(&TodoItem {
                id: "1".to_string(),
                title: "Buy oat milk".to_string(),
                completed: false,
                updated_at: "2025-06-15T11:00:00Z".to_string(),
            })
            .unwrap(),
        };

        let mut cmd = app.update(Event::SseEvent(msg), &mut model);

        assert_eq!(model.items[0].title, "Buy oat milk");
        assert_eq!(
            model.items[0].updated_at,
            "2025-06-15T11:00:00Z"
        );
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn sse_event_inserts_new_item() {
        let app = TodoApp;
        let mut model = seeded_model();

        let msg = SseMessage {
            event_type: "item_created".to_string(),
            data: serde_json::to_string(&make_item("99", "From server", false))
                .unwrap(),
        };

        let mut cmd = app.update(Event::SseEvent(msg), &mut model);

        assert_eq!(model.items.len(), 3);
        assert_eq!(model.items[2].title, "From server");
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn sse_delete_removes_item() {
        let app = TodoApp;
        let mut model = seeded_model();

        let msg = SseMessage {
            event_type: "item_deleted".to_string(),
            data: r#"{"id":"1"}"#.to_string(),
        };

        let mut cmd = app.update(Event::SseEvent(msg), &mut model);

        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "2");
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn lww_server_wins_on_equal_timestamp() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            items: vec![TodoItem {
                id: "1".to_string(),
                title: "Local title".to_string(),
                completed: false,
                updated_at: "2025-06-15T10:00:00Z".to_string(),
            }],
            ..Model::default()
        };

        let msg = SseMessage {
            event_type: "item_updated".to_string(),
            data: serde_json::to_string(&TodoItem {
                id: "1".to_string(),
                title: "Server title".to_string(),
                completed: false,
                updated_at: "2025-06-15T10:00:00Z".to_string(),
            })
            .unwrap(),
        };

        let _cmd = app.update(Event::SseEvent(msg), &mut model);

        assert_eq!(
            model.items[0].title, "Server title",
            "server should win on equal timestamps"
        );
    }

    #[test]
    fn lww_local_wins_on_later_timestamp() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            items: vec![TodoItem {
                id: "1".to_string(),
                title: "Local title".to_string(),
                completed: false,
                updated_at: "2025-06-15T11:00:00Z".to_string(),
            }],
            ..Model::default()
        };

        let msg = SseMessage {
            event_type: "item_updated".to_string(),
            data: serde_json::to_string(&TodoItem {
                id: "1".to_string(),
                title: "Server title".to_string(),
                completed: false,
                updated_at: "2025-06-15T10:00:00Z".to_string(),
            })
            .unwrap(),
        };

        let _cmd = app.update(Event::SseEvent(msg), &mut model);

        assert_eq!(
            model.items[0].title, "Local title",
            "local should win with later timestamp"
        );
    }

    #[test]
    fn connect_sse_sets_connecting_state() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::ConnectSse, &mut model);

        assert_eq!(model.sse_state, SseConnectionState::Connecting);
        let _sse_request =
            cmd.expect_one_effect().expect_server_sent_events();
    }

    #[test]
    fn saved_ok_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::Saved(Ok(None)), &mut model);
        assert!(cmd.is_done());
    }

    #[test]
    fn edit_title_with_blank_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::EditTitle("1".to_string(), "  ".to_string()),
            &mut model,
        );

        assert_eq!(model.items[0].title, "Buy milk");
        assert!(cmd.is_done());
    }

    #[test]
    fn edit_title_nonexistent_item_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::EditTitle("999".to_string(), "hello".to_string()),
            &mut model,
        );

        assert!(model.pending_ops.is_empty());
        assert!(cmd.is_done());
    }

    #[test]
    fn toggle_nonexistent_item_is_noop() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(
            Event::ToggleCompleted("999".to_string()),
            &mut model,
        );

        assert!(model.pending_ops.is_empty());
        assert!(cmd.is_done());
    }

    #[test]
    fn delete_nonexistent_item_does_not_queue_op() {
        let app = TodoApp;
        let mut model = seeded_model();
        let count_before = model.items.len();

        let mut cmd =
            app.update(Event::DeleteItem("999".to_string()), &mut model);

        assert_eq!(model.items.len(), count_before);
        assert!(model.pending_ops.is_empty());
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn clear_completed_with_none_completed_is_noop() {
        let app = TodoApp;
        let mut model = Model {
            page: Page::TodoList,
            items: vec![make_item("1", "A", false)],
            ..Model::default()
        };

        let mut cmd = app.update(Event::ClearCompleted, &mut model);

        assert_eq!(model.items.len(), 1);
        assert!(model.pending_ops.is_empty());
        cmd.expect_effect().expect_render();
    }

    #[test]
    fn start_retry_timer_requests_time_notification() {
        let app = TodoApp;
        let mut model = seeded_model();

        let mut cmd = app.update(Event::StartRetryTimer, &mut model);
        let _request = cmd.expect_one_effect().expect_time();
    }
}
