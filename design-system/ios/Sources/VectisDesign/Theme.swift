import SwiftUI

/// Bundles the full Vectis design system for SwiftUI environment injection.
///
/// Apply at the app root:
/// ```swift
/// @main
/// struct MyApp: App {
///     var body: some Scene {
///         WindowGroup {
///             ContentView()
///                 .vectisTheme()
///         }
///     }
/// }
/// ```
///
/// Access in any view:
/// ```swift
/// @Environment(\.vectisTheme) private var theme
/// Text("Hello").font(theme.typography.title)
/// ```
public struct VectisTheme: Sendable {
    public let colors: VectisColors.Type = VectisColors.self
    public let typography: VectisTypography.Type = VectisTypography.self
    public let spacing: VectisSpacing.Type = VectisSpacing.self
    public let cornerRadius: VectisCornerRadius.Type = VectisCornerRadius.self

    public init() {}
}

// MARK: - Environment Key

private struct VectisThemeKey: EnvironmentKey {
    static let defaultValue = VectisTheme()
}

extension EnvironmentValues {
    public var vectisTheme: VectisTheme {
        get { self[VectisThemeKey.self] }
        set { self[VectisThemeKey.self] = newValue }
    }
}

// MARK: - View Modifier

extension View {
    /// Injects the Vectis design system theme into the view hierarchy.
    public func vectisTheme() -> some View {
        environment(\.vectisTheme, VectisTheme())
    }
}
