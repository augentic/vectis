import Inject
import SharedTypes
import SwiftUI

struct TodoListScreen: View {
    let viewModel: TodoListView
    let onEvent: (Event) -> Void
    @ObserveInjection var inject

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                InputArea(newTitle: viewModel.newTitle, onEvent: onEvent)

                FilterBar(filter: viewModel.filter, onEvent: onEvent)

                ItemList(items: viewModel.items, onEvent: onEvent)

                Footer(
                    activeCount: Int(viewModel.activeCount),
                    hasCompleted: viewModel.hasCompleted,
                    onEvent: onEvent
                )

                StatusBar(
                    syncStatus: viewModel.syncStatus,
                    sseState: viewModel.sseState,
                    pendingCount: Int(viewModel.pendingCount)
                )
            }
            .navigationTitle("Todos")
            .navigationBarTitleDisplayMode(.large)
        }
        .enableInjection()
    }
}

// MARK: - Input Area

private struct InputArea: View {
    let newTitle: String
    let onEvent: (Event) -> Void
    @State private var text = ""
    @ObserveInjection var inject

    var body: some View {
        HStack(spacing: 8) {
            TextField("What needs to be done?", text: $text)
                .textFieldStyle(.roundedBorder)
                .onSubmit { submit() }
                .onChange(of: text) { _, newValue in
                    onEvent(.setNewTitle(newValue))
                }

            Button {
                submit()
            } label: {
                Image(systemName: "plus.circle.fill")
                    .font(.title2)
            }
            .disabled(text.trimmingCharacters(in: .whitespaces).isEmpty)
            .accessibilityLabel("Add todo")
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .onAppear { text = newTitle }
        .enableInjection()
    }

    private func submit() {
        let trimmed = text.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        let id = UUID().uuidString
        onEvent(.addTodo(id: id))
        text = ""
    }
}

// MARK: - Filter Bar

private struct FilterBar: View {
    let filter: Filter
    let onEvent: (Event) -> Void
    @ObserveInjection var inject

    var body: some View {
        Picker("Filter", selection: Binding(
            get: { filter },
            set: { onEvent(.setFilter($0)) }
        )) {
            Text("All").tag(Filter.all)
            Text("Active").tag(Filter.active)
            Text("Completed").tag(Filter.completed)
        }
        .pickerStyle(.segmented)
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .enableInjection()
    }
}

// MARK: - Item List

private struct ItemList: View {
    let items: [TodoItemView]
    let onEvent: (Event) -> Void
    @ObserveInjection var inject

    var body: some View {
        List(items, id: \.id) { item in
            TodoRow(item: item, onEvent: onEvent)
                .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                    Button(role: .destructive) {
                        let generator = UIImpactFeedbackGenerator(style: .medium)
                        generator.impactOccurred()
                        onEvent(.deleteTodo(item.id))
                    } label: {
                        Label("Delete", systemImage: "trash")
                    }
                }
        }
        .listStyle(.plain)
        .enableInjection()
    }
}

// MARK: - Todo Row

private struct TodoRow: View {
    let item: TodoItemView
    let onEvent: (Event) -> Void
    @State private var isEditing = false
    @State private var editText = ""
    @ObserveInjection var inject

    var body: some View {
        HStack(spacing: 12) {
            Button {
                let generator = UIImpactFeedbackGenerator(style: .light)
                generator.impactOccurred()
                onEvent(.toggleTodo(item.id))
            } label: {
                Image(systemName: item.completed ? "checkmark.circle.fill" : "circle")
                    .font(.title3)
                    .foregroundStyle(item.completed ? .blue : .secondary)
            }
            .buttonStyle(.plain)
            .accessibilityLabel(item.completed ? "Mark incomplete" : "Mark complete")

            if isEditing {
                TextField("Title", text: $editText)
                    .textFieldStyle(.roundedBorder)
                    .onSubmit {
                        let trimmed = editText.trimmingCharacters(in: .whitespaces)
                        if !trimmed.isEmpty {
                            onEvent(.editTitle(id: item.id, title: trimmed))
                        }
                        isEditing = false
                    }
            } else {
                Text(item.title)
                    .font(.body)
                    .strikethrough(item.completed)
                    .foregroundStyle(item.completed ? .secondary : .primary)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        editText = item.title
                        isEditing = true
                    }
            }
        }
        .padding(.vertical, 4)
        .enableInjection()
    }
}

// MARK: - Footer

private struct Footer: View {
    let activeCount: Int
    let hasCompleted: Bool
    let onEvent: (Event) -> Void
    @ObserveInjection var inject

    var body: some View {
        HStack {
            Text("\(activeCount) item\(activeCount == 1 ? "" : "s") left")
                .font(.caption)
                .foregroundStyle(.secondary)
                .contentTransition(.numericText())

            Spacer()

            if hasCompleted {
                Button("Clear completed") {
                    onEvent(.clearCompleted)
                }
                .font(.caption)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .enableInjection()
    }
}

// MARK: - Status Bar

private struct StatusBar: View {
    let syncStatus: SyncStatus
    let sseState: SseState
    let pendingCount: Int
    @ObserveInjection var inject

    var body: some View {
        HStack(spacing: 8) {
            statusIndicator

            if pendingCount > 0 {
                Text("\(pendingCount) pending")
                    .font(.caption2)
                    .contentTransition(.numericText())
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(.orange.opacity(0.2))
                    .clipShape(Capsule())
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 6)
        .frame(maxWidth: .infinity)
        .background(Color(.secondarySystemBackground))
        .enableInjection()
    }

    @ViewBuilder
    private var statusIndicator: some View {
        switch (syncStatus, sseState) {
        case (.syncing, _):
            Label("Syncing...", systemImage: "arrow.triangle.2.circlepath")
                .font(.caption2)
                .foregroundStyle(.blue)

        case (.offline, _):
            Label("Offline", systemImage: "circle.fill")
                .font(.caption2)
                .foregroundStyle(.red)

        case (.idle, .connected):
            Label("Synced", systemImage: "circle.fill")
                .font(.caption2)
                .foregroundStyle(.green)

        case (.idle, .connecting):
            Label("Connecting...", systemImage: "circle.fill")
                .font(.caption2)
                .foregroundStyle(.orange)

        case (.idle, .disconnected):
            Label("Disconnected", systemImage: "circle.fill")
                .font(.caption2)
                .foregroundStyle(.gray)
        }
    }
}

// MARK: - Previews

#Preview("With items") {
    TodoListScreen(
        viewModel: TodoListView(
            items: [
                TodoItemView(id: "1", title: "Buy groceries", completed: false),
                TodoItemView(id: "2", title: "Walk the dog", completed: true),
                TodoItemView(id: "3", title: "Write code", completed: false),
            ],
            newTitle: "",
            activeCount: 2,
            hasCompleted: true,
            filter: .all,
            syncStatus: .idle,
            sseState: .connected,
            pendingCount: 0
        ),
        onEvent: { _ in }
    )
}

#Preview("Empty") {
    TodoListScreen(
        viewModel: TodoListView(
            items: [],
            newTitle: "",
            activeCount: 0,
            hasCompleted: false,
            filter: .all,
            syncStatus: .idle,
            sseState: .disconnected,
            pendingCount: 0
        ),
        onEvent: { _ in }
    )
}

#Preview("Syncing") {
    TodoListScreen(
        viewModel: TodoListView(
            items: [
                TodoItemView(id: "1", title: "Pending task", completed: false),
            ],
            newTitle: "",
            activeCount: 1,
            hasCompleted: false,
            filter: .all,
            syncStatus: .syncing,
            sseState: .connected,
            pendingCount: 2
        ),
        onEvent: { _ in }
    )
}
