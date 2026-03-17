import Inject
import SwiftUI

struct LoadingScreen: View {
    @ObserveInjection var inject

    var body: some View {
        VStack(spacing: 12) {
            ProgressView()
                .controlSize(.large)
            Text("Loading...")
                .font(.body)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(.systemBackground))
        .enableInjection()
    }
}

#Preview {
    LoadingScreen()
}
