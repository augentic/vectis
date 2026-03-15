import Inject
import SwiftUI
import VectisDesign

@main
struct TodoApp: App {
    @StateObject private var core = Core()
    @ObserveInjection var inject

    var body: some Scene {
        WindowGroup {
            ContentView(core: core)
                .vectisTheme()
        }
    }
}
