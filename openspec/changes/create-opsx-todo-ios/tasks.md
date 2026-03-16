## 1. Project Scaffolding

- [ ] 1.1 Create `examples/opsx_todo/iOS/` directory structure with `Todo/` and `Todo/Views/` subdirectories
- [ ] 1.2 Generate `project.yml` with XcodeGen configuration: Shared, SharedTypes, VectisDesign, and Inject package references; iOS 17+ deployment target; Swift 6 strict concurrency; Debug-only Inject linker flags
- [ ] 1.3 Generate `Makefile` with three-phase build pipeline: `typegen` (SharedTypes codegen), `package` (cargo-swift XCFramework), `xcode` (xcodegen)

## 2. Core Bridge

- [ ] 2.1 Generate `Todo/Core.swift` with `@MainActor ObservableObject` Core class: process loop, `@Published var view`, and `update` method
- [ ] 2.2 Implement Render effect handler — deserialize ViewModel and update published view
- [ ] 2.3 Implement Http effect handler — execute requests via URLSession, return response bytes to core
- [ ] 2.4 Implement KeyValue effect handler — handle get/set/delete/exists/listKeys operations using file-based storage
- [ ] 2.5 Implement Time effect handler — schedule async timer via `Task.sleep` and notify core on completion
- [ ] 2.6 Implement ServerSentEvents effect handler — open URLSession async byte stream, parse SSE frames, dispatch events to core

## 3. Views

- [ ] 3.1 Generate `Todo/ContentView.swift` — switch on `core.view` with cases for Loading, TodoList, and Error ViewModel variants
- [ ] 3.2 Generate `Todo/Views/LoadingScreen.swift` — centered ProgressView with VectisDesign styling
- [ ] 3.3 Generate `Todo/Views/TodoListScreen.swift` — text input with Add button, scrollable item list with checkboxes/strikethrough/swipe-delete, filter tabs (All/Active/Completed), footer with active count, sync status dot, and Clear completed button
- [ ] 3.4 Generate `Todo/Views/ErrorScreen.swift` — centered error message with Retry button (when `can_retry` is true)
- [ ] 3.5 Wire all shell-facing Event dispatches: Navigate on launch, AddItem with generated ID, EditTitle, ToggleCompleted, DeleteItem, ClearCompleted, SetFilter

## 4. App Entry Point

- [ ] 4.1 Generate `Todo/TodoApp.swift` — `@main` struct with `@StateObject` Core, `.vectisTheme()` modifier, and Inject integration

## 5. Verification

- [ ] 5.1 Run `swiftformat` on all generated Swift files
- [ ] 5.2 Run `make typegen` and `make package` to generate Swift packages from the existing Crux core
- [ ] 5.3 Run `make xcode` to generate the Xcode project
- [ ] 5.4 Run `make build` to verify the project compiles for iOS Simulator
- [ ] 5.5 Fix any build errors and re-verify
