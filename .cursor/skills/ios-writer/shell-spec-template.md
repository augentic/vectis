# iOS Shell Specification: [App Name]

<!-- Fill out each section below. The ios-writer skill uses this file along
     with the Crux core's app.rs to generate the SwiftUI shell. Delete the
     guidance comments once you've filled in each section. -->

## Overview

<!-- One sentence: which Crux app is this shell for, and where does it live?
     Example: iOS shell for the TodoApp at examples/opsx_todo -->

## Target Directory

<!-- Where the iOS shell will be generated.
     Example: examples/opsx_todo/iOS -->

## Navigation Style

<!-- How the app navigates between views. Pick one:
     - single: One screen, no navigation (e.g., counter)
     - stack: NavigationStack with push/pop (e.g., list → detail)
     - tabs: TabView with bottom tabs (e.g., home + settings)
     Describe which views map to which tabs or stack levels. -->

## Screen Customizations

<!-- For each ViewModel variant / screen, describe any iOS-specific UI
     details that go beyond the core spec's User Interface section.
     These are hints for the ios-writer about layout and interaction.

     Example:

     ### TodoList Screen
     - Use a List with swipe-to-delete on each item
     - Add pull-to-refresh bound to Event::Refresh
     - Show sync status in a toolbar subtitle

     ### Error Screen
     - Use default error screen pattern (no customization needed) -->

## Platform Features

<!-- iOS-specific features to include. Remove rows that don't apply.

     | Feature | Details |
     |---|---|
     | Haptic feedback | On toggle actions |
     | Share sheet | For exporting data |
     | Widgets | Not needed |
     | Push notifications | Not needed |
     | App intents / Shortcuts | Not needed | -->

## Design System Overrides

<!-- Any design token overrides for this specific app. Leave empty to use
     the default VectisDesign tokens.

     Example:
     - Use `secondary` instead of `primary` for the main action button
     - Use `title3` for list item titles instead of `body` -->
