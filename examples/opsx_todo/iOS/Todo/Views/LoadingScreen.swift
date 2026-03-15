import Inject
import SwiftUI
import VectisDesign

struct LoadingScreen: View {
    @ObserveInjection var inject

    var body: some View {
        VStack(spacing: VectisSpacing.md) {
            ProgressView()
                .controlSize(.large)
            Text("Loading...")
                .font(VectisTypography.body)
                .foregroundStyle(VectisColors.onSurfaceSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(VectisColors.surface)
        .enableInjection()
    }
}

#Preview {
    LoadingScreen()
        .vectisTheme()
}
