## ADDED Requirements

### Requirement: Screen view for every ViewModel variant

The iOS shell SHALL render a dedicated screen view for each ViewModel variant
defined in `shared/src/app.rs`. The mapping is:

| ViewModel variant | Screen file | Content |
|---|---|---|
| `Loading` | `LoadingScreen.swift` | Centered `ProgressView` with "Loading..." label |
| `TodoList(TodoListView)` | `TodoListScreen.swift` | Full todo list UI (input, list, footer, filters) |
| `Error(ErrorView)` | `ErrorScreen.swift` | Error message with conditional "Retry" button |

`ContentView.swift` SHALL contain a `switch` on `core.view` with exactly one
case per ViewModel variant.

#### Scenario: All ViewModel variants have screen views

- **WHEN** the app's `app.rs` defines ViewModel variants `Loading`, `TodoList(TodoListView)`, and `Error(ErrorView)`
- **THEN** the iOS shell contains `LoadingScreen.swift`, `TodoListScreen.swift`, and `ErrorScreen.swift` under `Views/`
- **AND** `ContentView.swift` has a switch case for each variant

#### Scenario: Loading screen shows spinner

- **WHEN** `core.view` is `ViewModel.Loading`
- **THEN** the shell renders a centered `ProgressView` with "Loading..." text
- **AND** no interactive elements are present

#### Scenario: Error screen shows message and retry

- **WHEN** `core.view` is `ViewModel.Error(errorView)` with `canRetry == true`
- **THEN** the shell renders the error message and a "Retry" button
- **AND** tapping "Retry" dispatches `Event.retrySync`

### Requirement: Effect handler for every Effect variant

`Core.swift` SHALL handle all four Effect variants defined in `app.rs`:
`Render`, `Http`, `KeyValue`, and `ServerSentEvents`. Each effect handler
SHALL process the operation and send the result back to the core.

#### Scenario: Render effect updates the published view

- **WHEN** the core emits a `Render` effect
- **THEN** `Core.swift` deserializes the ViewModel and assigns it to `@Published var view`
- **AND** SwiftUI re-renders the active screen

#### Scenario: Http effect performs network request

- **WHEN** the core emits an `Http` effect with an `HttpRequest`
- **THEN** `Core.swift` performs the HTTP request via `URLSession`
- **AND** sends the `HttpResponse` (or error) back to the core

#### Scenario: KeyValue effect reads and writes local storage

- **WHEN** the core emits a `KeyValue` effect with a `Get` operation
- **THEN** `Core.swift` reads the value from `UserDefaults` and returns it
- **WHEN** the core emits a `KeyValue` effect with a `Set` operation
- **THEN** `Core.swift` writes the value to `UserDefaults`

#### Scenario: ServerSentEvents effect connects to SSE stream

- **WHEN** the core emits a `ServerSentEvents` effect with a `Connect` operation
- **THEN** `Core.swift` opens an async `URLSession.bytes` stream to the given URL
- **AND** parses incoming SSE frames and sends them back to the core
- **WHEN** the core emits a `ServerSentEvents` effect with a `Disconnect` operation
- **THEN** `Core.swift` cancels the active SSE stream task

### Requirement: Shell-facing events dispatched from UI

Every shell-facing Event variant (those without `#[serde(skip)]`) SHALL be
dispatched by at least one view through user interaction.

| Event | Dispatched from | Trigger |
|---|---|---|
| `Initialize` | `Core.init()` or `onAppear` | App launch |
| `Navigate(Route)` | N/A (single route) | — |
| `SetInput(String)` | `TodoListScreen` | Text field binding change |
| `AddTodo(String, String)` | `TodoListScreen` | "Add" button tap |
| `EditTitle(String, String, String)` | `TodoListScreen` | Inline edit commit |
| `ToggleCompleted(String, String)` | `TodoListScreen` | Checkbox tap |
| `DeleteTodo(String, String)` | `TodoListScreen` | Swipe delete action |
| `ClearCompleted(String)` | `TodoListScreen` | "Clear completed" button tap |
| `SetFilter(Filter)` | `TodoListScreen` | Filter tab selection |
| `RetrySync` | `TodoListScreen` / `ErrorScreen` | Retry button tap |
| `ConnectSse` | `Core.init()` or `onAppear` | Automatic on app start |
| `SseDisconnected` | `Core.swift` SSE handler | Stream error/close |

#### Scenario: Add todo dispatches event with ID and timestamp

- **WHEN** user types "Buy milk" and taps "Add"
- **THEN** the shell dispatches `Event.addTodo(generatedId, currentTimestamp)`
- **AND** the text field is cleared

#### Scenario: Toggle completed dispatches event

- **WHEN** user taps the checkbox on a todo item
- **THEN** the shell dispatches `Event.toggleCompleted(itemId, currentTimestamp)`

#### Scenario: Delete todo via swipe

- **WHEN** user swipes left on a todo item and taps delete
- **THEN** the shell dispatches `Event.deleteTodo(itemId, currentTimestamp)`

#### Scenario: Filter selection dispatches event

- **WHEN** user taps the "Active" filter tab
- **THEN** the shell dispatches `Event.setFilter(.active)`

### Requirement: Design system token usage

All visual styling in the iOS shell SHALL use `VectisDesign` tokens. No
hardcoded colors, font sizes, spacing values, or corner radii in view code.

#### Scenario: Colors use VectisColors

- **WHEN** any view renders a colored element
- **THEN** the color value comes from `VectisColors` (e.g., `VectisColors.primary`, `VectisColors.surface`)
- **AND** no `Color(red:)`, `Color("name")`, or hex values appear in view code

#### Scenario: Typography uses VectisTypography

- **WHEN** any view renders text with a font modifier
- **THEN** the font comes from `VectisTypography` (e.g., `VectisTypography.headline`)
- **AND** no `.font(.system(size:))` appears in view code (except for SF Symbol sizing)

#### Scenario: Spacing uses VectisSpacing

- **WHEN** any view applies padding or spacing
- **THEN** the value comes from `VectisSpacing` (e.g., `VectisSpacing.md`)
- **AND** no magic number padding values appear in view code

### Requirement: TodoList screen layout

The `TodoListScreen` SHALL render the todo list UI matching the app
specification's "Todo List" view description.

#### Scenario: Input area renders text field and add button

- **WHEN** `TodoListScreen` renders with a `TodoListView`
- **THEN** a text field bound to `inputText` appears at the top
- **AND** an "Add" button appears next to the text field

#### Scenario: Todo items render in scrollable list

- **WHEN** the `TodoListView` contains items
- **THEN** each item shows a checkbox, title text, and delete action
- **AND** completed items show strikethrough styling

#### Scenario: Footer shows active count and sync status

- **WHEN** `TodoListScreen` renders
- **THEN** the footer shows the `activeCount` text
- **AND** the footer shows the `syncStatus` text
- **AND** the "Clear completed" button appears only when `showClearCompleted` is true

#### Scenario: Filter tabs control visible items

- **WHEN** the filter tabs appear in the footer
- **THEN** "All", "Active", and "Completed" tabs are shown
- **AND** the active filter is visually highlighted

### Requirement: Build and project configuration

The iOS shell SHALL include a complete build configuration that compiles
without errors.

#### Scenario: XcodeGen generates a valid project

- **WHEN** `make setup` is run in the iOS directory
- **THEN** XcodeGen produces a valid `.xcodeproj` from `project.yml`
- **AND** the project references `VectisDesign` and `shared.xcodeproj`

#### Scenario: Build succeeds for simulator

- **WHEN** `make build` is run in the iOS directory
- **THEN** the project compiles for the iOS Simulator without errors

#### Scenario: Swift formatting passes

- **WHEN** `swiftformat --lint` is run on the generated Swift files
- **THEN** no formatting violations are reported

### Requirement: Accessibility and previews

All interactive elements without visible text labels SHALL have
`accessibilityLabel` modifiers, and all screen views SHALL include
`#Preview` blocks with sample data.

#### Scenario: Icon-only buttons have accessibility labels

- **WHEN** a button contains only an SF Symbol image (no Text)
- **THEN** the button or image has an `accessibilityLabel` describing the action

#### Scenario: Every screen view has a preview

- **WHEN** a screen view file exists in `Views/`
- **THEN** it contains a `#Preview` block with sample data for development

### Requirement: Hot reloading support

All generated views SHALL include Inject library integration for hot reloading
during development.

#### Scenario: Views include Inject boilerplate

- **WHEN** a view struct is generated
- **THEN** it imports `Inject`
- **AND** declares `@ObserveInjection var inject`
- **AND** applies `.enableInjection()` as the outermost body modifier

#### Scenario: Project configuration includes Inject

- **WHEN** `project.yml` is generated
- **THEN** it includes the `Inject` SPM package dependency
- **AND** Debug config includes `OTHER_LDFLAGS` with `-Xlinker -interposable`
