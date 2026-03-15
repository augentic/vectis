import Inject
import SharedTypes
import SwiftUI
import VectisDesign

struct TodoListScreen: View {
    let viewModel: TodoListView
    let onEvent: (Event) -> Void

    @State private var editingItemId: String?
    @State private var editText = ""

    @ObserveInjection var inject

    var body: some View {
        VStack(spacing: 0) {
            inputSection
            itemList
            footerSection
        }
        .background(VectisColors.surface)
        .enableInjection()
    }

    // MARK: - Input Section

    private var inputSection: some View {
        HStack(spacing: VectisSpacing.sm) {
            TextField("What needs to be done?", text: Binding(
                get: { viewModel.inputText },
                set: { onEvent(.setInput($0)) }
            ))
            .textFieldStyle(.roundedBorder)
            .font(VectisTypography.body)
            .onSubmit { addTodo() }

            Button {
                addTodo()
            } label: {
                Image(systemName: "plus.circle.fill")
                    .font(.system(size: 28))
            }
            .tint(VectisColors.primary)
            .disabled(viewModel.inputText.trimmingCharacters(in: .whitespaces).isEmpty)
            .accessibilityLabel("Add todo")
        }
        .padding(.horizontal, VectisSpacing.md)
        .padding(.vertical, VectisSpacing.sm)
        .background(VectisColors.surfaceSecondary)
    }

    // MARK: - Item List

    private var itemList: some View {
        List {
            ForEach(viewModel.items, id: \.id) { item in
                if editingItemId == item.id {
                    editRow(item: item)
                } else {
                    itemRow(item: item)
                }
            }
        }
        .listStyle(.plain)
    }

    private func itemRow(item: TodoItemView) -> some View {
        HStack(spacing: VectisSpacing.sm) {
            Button {
                onEvent(.toggleCompleted(item.id, currentTimestamp()))
            } label: {
                Image(systemName: item.completed ? "checkmark.circle.fill" : "circle")
                    .foregroundStyle(
                        item.completed ? VectisColors.primary : VectisColors.onSurfaceSecondary
                    )
                    .font(.system(size: 22))
            }
            .buttonStyle(.plain)
            .accessibilityLabel(item.completed ? "Mark incomplete" : "Mark complete")

            Text(item.title)
                .font(VectisTypography.body)
                .foregroundStyle(VectisColors.onSurface)
                .strikethrough(item.completed)
                .opacity(item.completed ? 0.6 : 1.0)
                .frame(maxWidth: .infinity, alignment: .leading)
                .contentShape(Rectangle())
                .onTapGesture {
                    editingItemId = item.id
                    editText = item.title
                }
        }
        .swipeActions(edge: .trailing, allowsFullSwipe: true) {
            Button(role: .destructive) {
                onEvent(.deleteTodo(item.id, currentTimestamp()))
            } label: {
                Label("Delete", systemImage: "trash")
            }
        }
    }

    private func editRow(item: TodoItemView) -> some View {
        HStack(spacing: VectisSpacing.sm) {
            TextField("Edit title", text: $editText)
                .textFieldStyle(.roundedBorder)
                .font(VectisTypography.body)
                .onSubmit { commitEdit(itemId: item.id) }

            Button("Done") {
                commitEdit(itemId: item.id)
            }
            .font(VectisTypography.subheadline)
            .tint(VectisColors.primary)

            Button("Cancel") {
                editingItemId = nil
            }
            .font(VectisTypography.subheadline)
            .tint(VectisColors.onSurfaceSecondary)
        }
    }

    // MARK: - Footer

    private var footerSection: some View {
        VStack(spacing: VectisSpacing.sm) {
            filterTabs

            HStack {
                Text(viewModel.activeCount)
                    .font(VectisTypography.caption)
                    .foregroundStyle(VectisColors.onSurfaceSecondary)

                Spacer()

                syncStatusLabel

                Spacer()

                if viewModel.showClearCompleted {
                    Button("Clear completed") {
                        onEvent(.clearCompleted(currentTimestamp()))
                    }
                    .font(VectisTypography.caption)
                    .tint(VectisColors.error)
                }
            }
            .padding(.horizontal, VectisSpacing.md)
            .padding(.bottom, VectisSpacing.sm)
        }
        .background(VectisColors.surfaceSecondary)
    }

    private var filterTabs: some View {
        HStack(spacing: VectisSpacing.xs) {
            filterButton("All", filter: .all)
            filterButton("Active", filter: .active)
            filterButton("Completed", filter: .completed)
        }
        .padding(.horizontal, VectisSpacing.md)
        .padding(.top, VectisSpacing.sm)
    }

    private func filterButton(_ title: String, filter: Filter) -> some View {
        Button(title) {
            onEvent(.setFilter(filter))
        }
        .font(VectisTypography.subheadline)
        .padding(.horizontal, VectisSpacing.sm)
        .padding(.vertical, VectisSpacing.xs)
        .background(
            viewModel.filter == filter
                ? VectisColors.primary.opacity(0.15)
                : Color.clear
        )
        .foregroundStyle(
            viewModel.filter == filter
                ? VectisColors.primary
                : VectisColors.onSurfaceSecondary
        )
        .clipShape(RoundedRectangle(cornerRadius: VectisCornerRadius.sm))
    }

    private var syncStatusLabel: some View {
        HStack(spacing: VectisSpacing.xs) {
            if !viewModel.pendingCount.isEmpty {
                Circle()
                    .fill(syncStatusColor)
                    .frame(width: 8, height: 8)
            }
            if viewModel.syncStatus == "Offline" {
                Button {
                    onEvent(.retrySync)
                } label: {
                    Label(viewModel.syncStatus, systemImage: "arrow.triangle.2.circlepath")
                        .font(VectisTypography.caption)
                        .foregroundStyle(VectisColors.error)
                }
                .buttonStyle(.plain)
            } else {
                Text(viewModel.syncStatus)
                    .font(VectisTypography.caption)
                    .foregroundStyle(VectisColors.onSurfaceSecondary)
            }
        }
    }

    private var syncStatusColor: Color {
        switch viewModel.syncStatus {
        case "Syncing":
            VectisColors.secondary
        case "Offline":
            VectisColors.error
        default:
            VectisColors.primary
        }
    }

    // MARK: - Actions

    private func addTodo() {
        let trimmed = viewModel.inputText.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        onEvent(.addTodo(generateId(), currentTimestamp()))
    }

    private func commitEdit(itemId: String) {
        let trimmed = editText.trimmingCharacters(in: .whitespaces)
        if !trimmed.isEmpty {
            onEvent(.editTitle(itemId, trimmed, currentTimestamp()))
        }
        editingItemId = nil
    }

    private func generateId() -> String {
        UUID().uuidString.lowercased()
    }

    private func currentTimestamp() -> String {
        ISO8601DateFormatter().string(from: Date())
    }
}

#Preview {
    TodoListScreen(
        viewModel: TodoListView(
            items: [
                TodoItemView(id: "1", title: "Buy groceries", completed: false),
                TodoItemView(id: "2", title: "Walk the dog", completed: true),
                TodoItemView(id: "3", title: "Write code", completed: false),
            ],
            inputText: "",
            activeCount: "2 items left",
            pendingCount: "",
            syncStatus: "Idle",
            filter: .all,
            showClearCompleted: true
        ),
        onEvent: { _ in }
    )
    .vectisTheme()
}
