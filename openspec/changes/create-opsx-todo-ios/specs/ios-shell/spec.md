## ADDED Requirements

### Requirement: Core bridge processes all effects
The iOS shell SHALL implement a `Core.swift` bridge that handles every `Effect` variant defined in the Crux core: Render, Http, KeyValue, Time, and ServerSentEvents.

#### Scenario: Render effect updates the published view
- **WHEN** the core emits a `Render` effect
- **THEN** Core.swift SHALL deserialize the ViewModel and update the `@Published var view` property

#### Scenario: HTTP effect performs network request
- **WHEN** the core emits an `Http` effect
- **THEN** Core.swift SHALL execute the HTTP request via URLSession and send the response back to the core

#### Scenario: KeyValue effect persists data
- **WHEN** the core emits a `KeyValue` effect with a set, get, delete, exists, or listKeys operation
- **THEN** Core.swift SHALL perform the operation using local file storage and return the result to the core

#### Scenario: Time effect schedules a timer
- **WHEN** the core emits a `Time` effect
- **THEN** Core.swift SHALL use `Task.sleep` to wait the requested duration and notify the core when the timer fires

#### Scenario: ServerSentEvents effect opens SSE stream
- **WHEN** the core emits a `ServerSentEvents` effect
- **THEN** Core.swift SHALL open an async URLSession connection, parse incoming SSE frames, and dispatch each event back to the core

### Requirement: ContentView renders all ViewModel variants
The iOS shell SHALL display a different screen for each ViewModel variant returned by the core.

#### Scenario: Loading state shows activity indicator
- **WHEN** the core's ViewModel is `Loading`
- **THEN** ContentView SHALL display a centered `ProgressView`

#### Scenario: TodoList state shows the main screen
- **WHEN** the core's ViewModel is `TodoList(TodoListView)`
- **THEN** ContentView SHALL display the TodoListScreen with the view model's data

#### Scenario: Error state shows error message with retry
- **WHEN** the core's ViewModel is `Error(ErrorView)`
- **THEN** ContentView SHALL display the ErrorScreen with the error message and a retry button (if `can_retry` is true)

### Requirement: TodoListScreen displays items and controls
The TodoListScreen SHALL render all elements of the `TodoListView` data: item list, input field, filter tabs, sync status, and action buttons.

#### Scenario: Item list displays filtered items
- **WHEN** the TodoListView contains items
- **THEN** the screen SHALL display each item with a checkbox, title, and delete action

#### Scenario: Completed items show strikethrough
- **WHEN** an item's `completed` field is true
- **THEN** the item's title SHALL be rendered with strikethrough styling

#### Scenario: Add item input is present
- **WHEN** the TodoListScreen is displayed
- **THEN** the screen SHALL include a text input field and an "Add" button at the top

#### Scenario: Filter tabs control visible items
- **WHEN** the user taps a filter tab (All, Active, Completed)
- **THEN** the shell SHALL dispatch `SetFilter` with the selected filter value

#### Scenario: Sync status indicator is visible
- **WHEN** the TodoListScreen is displayed
- **THEN** a footer SHALL show the sync status (synced/pending/offline) with a colored indicator dot and detail text

#### Scenario: Clear completed button appears when applicable
- **WHEN** `has_completed` is true
- **THEN** a "Clear completed" button SHALL be visible in the footer

### Requirement: Shell dispatches all shell-facing events
The iOS shell SHALL dispatch every shell-facing Event variant to the core in response to user interactions.

#### Scenario: AddItem event dispatched on add
- **WHEN** the user enters text and taps "Add"
- **THEN** the shell SHALL generate a unique ID and dispatch `AddItem(id, title)` to the core

#### Scenario: EditTitle event dispatched on edit
- **WHEN** the user edits an item's title
- **THEN** the shell SHALL dispatch `EditTitle(id, newTitle)` to the core

#### Scenario: ToggleCompleted event dispatched on checkbox tap
- **WHEN** the user taps an item's checkbox
- **THEN** the shell SHALL dispatch `ToggleCompleted(id)` to the core

#### Scenario: DeleteItem event dispatched on delete action
- **WHEN** the user performs a delete action (swipe or button) on an item
- **THEN** the shell SHALL dispatch `DeleteItem(id)` to the core

#### Scenario: ClearCompleted event dispatched on button tap
- **WHEN** the user taps the "Clear completed" button
- **THEN** the shell SHALL dispatch `ClearCompleted` to the core

#### Scenario: Navigate event dispatched on launch
- **WHEN** the app launches
- **THEN** the shell SHALL dispatch `Navigate(TodoList)` to the core to trigger initialization

### Requirement: Build pipeline produces runnable app
The iOS shell SHALL include a Makefile and project.yml that produce a buildable Xcode project.

#### Scenario: Typegen generates SharedTypes
- **WHEN** `make typegen` is run
- **THEN** the SharedTypes Swift package SHALL be generated at `generated/swift/SharedTypes/`

#### Scenario: Package builds XCFramework
- **WHEN** `make package` is run
- **THEN** the Shared Swift package with XCFramework SHALL be produced at `generated/swift/Shared/`

#### Scenario: Xcode project is generated
- **WHEN** `make xcode` is run
- **THEN** XcodeGen SHALL produce a valid `.xcodeproj` from `project.yml`

### Requirement: Design system tokens used for all styling
The iOS shell SHALL use VectisDesign tokens for colors, typography, spacing, and corner radii. No hardcoded style values.

#### Scenario: Colors come from VectisColors
- **WHEN** any view renders a colored element
- **THEN** the color SHALL reference a `VectisColors` token

#### Scenario: Typography comes from VectisTypography
- **WHEN** any view renders text
- **THEN** the font SHALL reference a `VectisTypography` token

### Requirement: Hot reloading support via Inject
Every SwiftUI view SHALL include Inject boilerplate for InjectionIII hot reloading.

#### Scenario: View includes Inject integration
- **WHEN** any view struct is defined
- **THEN** it SHALL include `import Inject`, `@ObserveInjection var inject`, and `.enableInjection()` as the outermost modifier
