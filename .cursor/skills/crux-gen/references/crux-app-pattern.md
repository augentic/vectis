# Crux App Pattern (0.17+ API)

This reference describes the types and trait implementation for a Crux core application
targeting the 0.17+ API on the master branch.

## App Trait

The `App` trait is the central interface. It has four associated types and two methods:

```rust
use crux_core::{App, Command};

#[derive(Default)]
pub struct MyApp;

impl App for MyApp {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model) -> Command {
        // handle events, mutate model, return side-effect commands
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        // map model to view model (pure function, no side effects)
    }
}
```

Key rules:
- The app struct must derive `Default` so Crux can construct it.
- `update()` returns `Command` (no generic parameters in 0.17+).
- `view()` is a pure function -- it reads model and returns a view model.
- There is no `Capabilities` associated type or `caps` parameter.

## Model

The `Model` holds all internal application state. It is never sent to the shell.

```rust
#[derive(Default)]
pub struct Model {
    count: isize,
    items: Vec<Item>,
    loading: bool,
}
```

Rules:
- Must implement `Default` to define initial state.
- Fields are `pub(crate)` or private -- never `pub` (they don't leave the core).
- Use newtypes for domain identifiers: `struct ItemId(String)`.
- Use enums for known value sets: `enum Filter { All, Active, Completed }`.
- Complex nested state is fine -- the model is a tree.

## Event

Events are the input to the core. They come from two sources:

1. **Shell-facing** -- triggered by user interaction in the UI, sent across FFI.
   These must be serializable.
2. **Internal** -- used as callbacks from side-effects. Marked with `#[serde(skip)]`
   and `#[facet(skip)]` so they cannot be sent from the shell.

```rust
use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Event {
    // Shell-facing events (user actions)
    Increment,
    Decrement,
    Reset,
    FetchData,

    // Internal events (effect callbacks)
    #[serde(skip)]
    #[facet(skip)]
    DataFetched(#[facet(opaque)] crux_http::Result<crux_http::Response<MyData>>),
}
```

Rules:
- Derive `Facet, Serialize, Deserialize` for FFI compatibility.
- Add `#[repr(C)]` for `facet` enum layout.
- Internal variants with non-serializable payloads (like `crux_http::Result`) must have
  `#[serde(skip)]` and `#[facet(skip)]`.
- Mark opaque fields inside skipped variants with `#[facet(opaque)]`.
- Enum tuple variants double as constructor functions for `then_send`:
  `Http::get(url).build().then_send(Event::DataFetched)`.

## ViewModel

The ViewModel is what the shell renders. It crosses the FFI boundary so it must be
fully serializable and have type generation support.

```rust
#[derive(Facet, Serialize, Deserialize, Debug, Clone, Default)]
pub struct ViewModel {
    pub count: String,
    pub items: Vec<ItemView>,
    pub is_loading: bool,
}
```

Rules:
- Derive `Facet, Serialize, Deserialize, Clone, Debug, Default`.
- All fields are `pub` (the shell reads them).
- Use `String` for formatted display values -- formatting logic belongs in the core's
  `view()` function, not in the shell.
- Use simple types the shell can easily consume. Avoid complex enums in the view model
  when a bool or string suffices.
- The ViewModel is computed fresh on each `view()` call; it is not incrementally updated.

## Effect

The `Effect` enum declares which side-effects the app can request from the shell.
Each variant wraps the `Operation` type from a capability.

```rust
use crux_core::{
    macros::effect,
    render::RenderOperation,
};
use crux_http::HttpRequest;
use crux_kv::KeyValueOperation;

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Http(HttpRequest),
    KeyValue(KeyValueOperation),
}
```

Rules:
- Annotate with `#[effect(facet_typegen)]` for type generation support.
- Derive `Debug`.
- Always include `Render(RenderOperation)` -- every app needs UI updates.
- Add one variant per capability used. The variant name is arbitrary but conventionally
  matches the capability name.
- The macro generates helper methods like `expect_render()`, `expect_http()`, etc.
  on effect types for use in tests.

## Type Aliases for Capabilities

When using published capabilities, define a type alias that binds the capability
to your app's `Effect` and `Event` types:

```rust
type Http = crux_http::Http<Effect, Event>;
type KeyValue = crux_kv::KeyValue<Effect, Event>;
```

Then use them in `update()`:

```rust
Http::get(API_URL).expect_json().build().then_send(Event::DataFetched)
```

## Supporting Types

Domain types used across Model, Event, and ViewModel:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemView {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}
```

Rules:
- Types that cross the FFI boundary (in Event or ViewModel) must derive `Facet`.
- Types only used internally (Model-only) do not need `Facet` or `Serialize`.
- Enums used in FFI types need `#[repr(C)]`.
- Use `#[default]` on the default variant of enums that derive `Default`.

## The `view()` Function

The `view()` function maps Model to ViewModel. It is a pure function with no
side effects. All formatting and presentation logic belongs here.

```rust
fn view(&self, model: &Self::Model) -> Self::ViewModel {
    ViewModel {
        count: format!("Count is: {}", model.count),
        items: model
            .items
            .iter()
            .filter(|item| match model.filter {
                Filter::All => true,
                Filter::Active => !item.completed,
                Filter::Completed => item.completed,
            })
            .map(|item| ItemView {
                id: item.id.clone(),
                title: item.title.clone(),
                completed: item.completed,
            })
            .collect(),
        is_loading: model.loading,
    }
}
```

## The `update()` Function

The `update()` function handles events, mutates the model, and returns commands.
Every match arm must return a `Command`.

```rust
fn update(&self, event: Event, model: &mut Model) -> Command {
    match event {
        Event::Increment => {
            model.count += 1;
            render::render()
        }
        Event::FetchData => {
            model.loading = true;
            render::render().and(
                Http::get(API_URL)
                    .expect_json()
                    .build()
                    .then_send(Event::DataFetched),
            )
        }
        Event::DataFetched(Ok(mut response)) => {
            model.data = response.take_body().unwrap();
            model.loading = false;
            render::render()
        }
        Event::DataFetched(Err(_)) => {
            model.loading = false;
            model.error = Some("Failed to fetch data".to_string());
            render::render()
        }
    }
}
```

Use `Command::done()` when no side-effects are needed and no render is required
(rare -- usually you want at least `render::render()`).
