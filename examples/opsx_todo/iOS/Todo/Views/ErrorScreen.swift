import Inject
import SharedTypes
import SwiftUI
import VectisDesign

struct ErrorScreen: View {
    let viewModel: ErrorView
    let onEvent: (Event) -> Void
    @ObserveInjection var inject

    var body: some View {
        VStack(spacing: VectisSpacing.lg) {
            Spacer()

            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 56))
                .foregroundStyle(VectisColors.error)
                .accessibilityHidden(true)

            Text(viewModel.message)
                .font(VectisTypography.body)
                .foregroundStyle(VectisColors.onSurface)
                .multilineTextAlignment(.center)
                .padding(.horizontal, VectisSpacing.xl)

            if viewModel.canRetry {
                Button("Try Again") {
                    onEvent(.retrySync)
                }
                .buttonStyle(.borderedProminent)
                .tint(VectisColors.primary)
            }

            Spacer()
        }
        .frame(maxWidth: .infinity)
        .background(VectisColors.surface)
        .enableInjection()
    }
}

#Preview("With retry") {
    ErrorScreen(
        viewModel: ErrorView(
            message: "Failed to load data. Please try again.",
            canRetry: true
        ),
        onEvent: { _ in }
    )
    .vectisTheme()
}

#Preview("Without retry") {
    ErrorScreen(
        viewModel: ErrorView(
            message: "An unexpected error occurred.",
            canRetry: false
        ),
        onEvent: { _ in }
    )
    .vectisTheme()
}
