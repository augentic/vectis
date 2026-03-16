import Inject
import SharedTypes
import SwiftUI

struct ErrorScreen: View {
    let viewModel: ErrorView
    let onEvent: (Event) -> Void
    @ObserveInjection var inject

    var body: some View {
        VStack(spacing: 24) {
            Spacer()

            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 56))
                .foregroundStyle(.red)
                .accessibilityHidden(true)

            Text(viewModel.message)
                .font(.body)
                .foregroundStyle(.primary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 32)

            if viewModel.canRetry {
                Button("Try Again") {
                    onEvent(.navigate(.todoList))
                }
                .buttonStyle(.borderedProminent)
            }

            Spacer()
        }
        .frame(maxWidth: .infinity)
        .background(Color(.systemBackground))
        .enableInjection()
    }
}

#Preview("With retry") {
    ErrorScreen(
        viewModel: ErrorView(
            message: "Failed to connect to server. Please check your connection.",
            canRetry: true
        ),
        onEvent: { _ in }
    )
}

#Preview("Without retry") {
    ErrorScreen(
        viewModel: ErrorView(
            message: "An unexpected error occurred.",
            canRetry: false
        ),
        onEvent: { _ in }
    )
}
