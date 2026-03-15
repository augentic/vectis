import Inject
import SharedTypes
import SwiftUI
import VectisDesign

struct ContentView: View {
    @ObservedObject var core: Core
    @ObserveInjection var inject

    var body: some View {
        Group {
            switch core.view {
            case .loading:
                LoadingScreen()
            case let .todoList(viewModel):
                TodoListScreen(viewModel: viewModel) { event in
                    core.update(event)
                }
            case let .error(viewModel):
                ErrorScreen(viewModel: viewModel) { event in
                    core.update(event)
                }
            }
        }
        .enableInjection()
    }
}
