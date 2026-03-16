import Inject
import SharedTypes
import SwiftUI

struct ContentView: View {
    @ObservedObject var core: Core
    @ObserveInjection var inject

    var body: some View {
        Group {
            switch core.view {
            case .loading:
                LoadingScreen()
            case let .error(viewModel):
                ErrorScreen(viewModel: viewModel) { event in
                    core.update(event)
                }
            case let .todoList(viewModel):
                TodoListScreen(viewModel: viewModel) { event in
                    core.update(event)
                }
            }
        }
        .enableInjection()
    }
}
