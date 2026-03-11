import SwiftUI

// MARK: - Semantic Colors
// Generated from design-system/tokens.yaml — do not edit manually.

public enum VectisColors {
    public static let primary = Color(light: "#007AFF", dark: "#0A84FF")
    public static let primaryContainer = Color(light: "#D6E4FF", dark: "#003A70")
    public static let onPrimary = Color(light: "#FFFFFF", dark: "#FFFFFF")
    public static let onPrimaryContainer = Color(light: "#001D36", dark: "#D6E4FF")

    public static let secondary = Color(light: "#5856D6", dark: "#5E5CE6")
    public static let secondaryContainer = Color(light: "#E8E0FF", dark: "#2F2D6E")
    public static let onSecondary = Color(light: "#FFFFFF", dark: "#FFFFFF")
    public static let onSecondaryContainer = Color(light: "#1C1B33", dark: "#E8E0FF")

    public static let surface = Color(light: "#FFFFFF", dark: "#1C1C1E")
    public static let surfaceSecondary = Color(light: "#F2F2F7", dark: "#2C2C2E")
    public static let onSurface = Color(light: "#000000", dark: "#FFFFFF")
    public static let onSurfaceSecondary = Color(light: "#3C3C43", dark: "#EBEBF5")

    public static let error = Color(light: "#FF3B30", dark: "#FF453A")
    public static let onError = Color(light: "#FFFFFF", dark: "#FFFFFF")

    public static let outline = Color(light: "#C6C6C8", dark: "#38383A")
    public static let shadow = Color(light: "#000000", dark: "#000000")
}

// MARK: - Color Initializer from Hex

extension Color {
    init(light: String, dark: String) {
        self.init(uiColor: UIColor { traits in
            traits.userInterfaceStyle == .dark
                ? UIColor(hex: dark)
                : UIColor(hex: light)
        })
    }
}

extension UIColor {
    convenience init(hex: String) {
        let hex = hex.trimmingCharacters(in: .init(charactersIn: "#"))
        var rgb: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&rgb)
        self.init(
            red: CGFloat((rgb >> 16) & 0xFF) / 255,
            green: CGFloat((rgb >> 8) & 0xFF) / 255,
            blue: CGFloat(rgb & 0xFF) / 255,
            alpha: 1
        )
    }
}
