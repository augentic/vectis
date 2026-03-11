# Vectis Design System

Platform-agnostic design specification for Vectis applications. Each platform
shell (iOS, Android, Web) implements these tokens using native APIs.

## Color System

Colors use semantic names that describe purpose, not appearance. Every color
has light and dark variants; the shell selects the appropriate variant based
on the system appearance setting.

### Roles

| Role | Purpose |
|------|---------|
| `primary` | Interactive elements: buttons, links, toggles, selected states |
| `primaryContainer` | Tinted backgrounds behind primary-colored content |
| `onPrimary` | Text/icons on top of `primary` surfaces |
| `onPrimaryContainer` | Text/icons on top of `primaryContainer` surfaces |
| `secondary` | Less prominent interactive elements, secondary buttons |
| `secondaryContainer` | Tinted backgrounds behind secondary content |
| `onSecondary` | Text/icons on top of `secondary` surfaces |
| `onSecondaryContainer` | Text/icons on top of `secondaryContainer` surfaces |
| `surface` | Default background for screens and cards |
| `surfaceSecondary` | Grouped or inset backgrounds (e.g., form sections) |
| `onSurface` | Primary text and icons on `surface` |
| `onSurfaceSecondary` | Secondary text, captions, placeholders |
| `error` | Error indicators, destructive actions |
| `onError` | Text/icons on top of `error` surfaces |
| `outline` | Borders, dividers, subtle separators |
| `shadow` | Drop shadows (opacity varies by elevation) |

### Usage Rules

- Never reference hex values directly in views; always use token names.
- For disabled states, apply 38% opacity to the normal color.
- Destructive actions use `error` as primary color, not `primary`.

## Typography Scale

Type styles are defined as a named scale. Each entry specifies a relative
size (in points for iOS, sp for Android, rem for Web) and a weight.

| Style | iOS Size (pt) | Weight | Usage |
|-------|---------------|--------|-------|
| `largeTitle` | 34 | Bold | Screen titles, hero text |
| `title` | 28 | Bold | Section headers |
| `title2` | 22 | Bold | Sub-section headers |
| `title3` | 20 | Semibold | Card titles, group labels |
| `headline` | 17 | Semibold | Emphasized body text, list headers |
| `body` | 17 | Regular | Default readable text |
| `callout` | 16 | Regular | Supporting explanatory text |
| `subheadline` | 15 | Regular | Metadata, timestamps |
| `footnote` | 13 | Regular | Captions, legal text |
| `caption` | 12 | Regular | Labels on badges, tags |
| `caption2` | 11 | Regular | Fine print |

### Usage Rules

- Body text is the default. Only deviate when hierarchy demands it.
- Do not use more than three type styles on a single screen.
- Respect dynamic type / accessibility text scaling on all platforms.

## Spacing Scale

A named geometric scale for padding, margins, and gaps. All values are
in logical points (density-independent).

| Token | Value (pt) | Usage |
|-------|------------|-------|
| `xxs` | 2 | Hairline gaps, icon-to-label spacing |
| `xs` | 4 | Tight internal padding |
| `sm` | 8 | Standard internal padding, small gaps |
| `md` | 16 | Default padding, list item spacing |
| `lg` | 24 | Section spacing, card padding |
| `xl` | 32 | Large section gaps |
| `xxl` | 48 | Screen-edge margins on tablets, hero spacing |

### Usage Rules

- Use `md` as the default spacing for most layout decisions.
- Screen-edge horizontal padding is `md` on phone, `xl` on tablet.
- Vertical spacing between sections is `lg`.

## Corner Radius Scale

| Token | Value (pt) | Usage |
|-------|------------|-------|
| `none` | 0 | Sharp corners (dividers, full-width elements) |
| `sm` | 4 | Subtle rounding (tags, badges) |
| `md` | 8 | Cards, text fields, buttons |
| `lg` | 12 | Modals, bottom sheets |
| `xl` | 16 | Large cards, image containers |
| `full` | 9999 | Circles, pills |

## Iconography

- Use SF Symbols on iOS, Material Icons on Android, Lucide on Web.
- Icon size matches the type style it appears next to (e.g., body text
  uses 17pt icons).
- Use `onSurface` for default icon color; `primary` for interactive icons.

## Platform-Specific Guidance

### iOS (SwiftUI)

- Map color tokens to `Color` via asset catalogs or programmatic
  `Color(light:dark:)` extensions.
- Map typography tokens to `Font.system(size:weight:)` with
  `.dynamicTypeSize(...)` support.
- Use `@Environment(\.colorScheme)` to adapt; prefer adaptive colors
  that handle this automatically.
- Navigation follows iOS conventions: `NavigationStack`, tab bars,
  swipe-back gestures.
- Safe area insets are handled by SwiftUI layout system; do not add
  manual padding for notch/home indicator.

### Android (future)

- Map color tokens to Material 3 `ColorScheme`.
- Map typography tokens to `TextStyle` / Material Typography.
- Use Jetpack Compose theming (`MaterialTheme`).

### Web (future)

- Map color tokens to CSS custom properties (`--color-primary`).
- Map typography tokens to CSS classes or utility scale.
- Use `prefers-color-scheme` media query for light/dark.
