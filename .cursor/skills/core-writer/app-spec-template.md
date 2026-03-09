# App Specification: [App Name]

<!-- Fill out each section below. The core-writer skill uses this file to generate
     your Crux shared crate. Delete the guidance comments once you've filled
     in each section. -->

## Overview

<!-- One or two sentences describing the purpose of the application. -->

## Features

<!-- List every feature the user can interact with. Each feature should describe
     a concrete user action and the expected outcome.

     Example:
     - **Add item** -- user enters text and taps "Add"; a new item appears in the list.
     - **Delete item** -- user swipes an item; it is removed from the list.
     - **Toggle complete** -- user taps an item; it toggles between active and completed. -->

- **Feature name** -- description of what happens.

## Data Model

<!-- Describe the state the application needs to track internally. Think about
     what data exists even when the UI is not visible.

     Example:
     - A list of to-do items, each with an id, title, and completed flag.
     - The currently selected filter (all, active, completed).
     - Whether a network request is in flight (loading indicator). -->

## User Interface

<!-- Describe what the user sees on each view. Focus on the data displayed,
     not visual styling. Organise by view -- use the view names from the
     Views section as sub-headings.

     Example:
     ### Item List
     - A text input field and an "Add" button at the top.
     - A list of items showing title and a completion checkbox.
     - A count of remaining active items ("3 items left").
     - Filter buttons: All, Active, Completed. -->

## Views

<!-- List every distinct screen or page the user can see. Each view is a
     self-contained UI state managed by the core. The shell pattern-matches
     on the view model variant to decide which screen to render.

     Include at least one view. For single-screen apps, list a Loading view
     (shown while initial data loads) and the main view. Consider whether
     blocking error conditions (failed initialization, auth failure) need
     a dedicated error view with a retry affordance.

     Recoverable errors that don't block the whole screen (e.g. a failed
     HTTP request, going offline) are better handled as fields within a
     page's view data -- they don't need their own view.

     Example:
     - **Loading** -- a spinner shown while initial data loads from local storage.
     - **Error** -- a full-screen error with a message and retry button, shown
       when initial data loading fails.
     - **Item List** -- the main list view with items, filters, and action bar.
     - **Item Detail** -- shows a single item with edit controls. Navigated to
       by tapping an item in the list; returns to Item List on save or cancel. -->

- **View name** -- what the user sees and when they see it.

## Capabilities

<!-- Indicate which external capabilities the app needs. Remove any rows that
     don't apply; add detail where they do.

     | Capability | Needed? | Details |
     |---|---|---|
     | **HTTP** | Yes / No | e.g. "Fetches weather from `GET /api/weather?city={city}`" |
     | **Key-Value storage** | Yes / No | e.g. "Persists the list of notes locally" |
     | **Timer / Time** | Yes / No | e.g. "Auto-refreshes data every 30 seconds" |
     | **Server-Sent Events** | Yes / No | e.g. "Subscribes to live score updates" |
     | **Platform detection** | Yes / No | e.g. "Adjusts layout for mobile vs desktop" | -->

| Capability | Needed? | Details |
|---|---|---|
| **HTTP** | | |
| **Key-Value storage** | | |
| **Timer / Time** | | |
| **Server-Sent Events** | | |
| **Platform detection** | | |

## API Details

<!-- If the app uses HTTP, describe the endpoints it calls. Include method, URL
     pattern, request body (if any), and the shape of the response.

     Remove this section entirely if the app has no HTTP capability.

     Example:
     ### GET /api/todos
     Returns all to-do items.
     ```json
     [{ "id": "abc", "title": "Buy milk", "completed": false }]
     ```

     ### POST /api/todos
     Creates a new to-do item.
     Request: `{ "title": "Buy milk" }`
     Response: `{ "id": "abc", "title": "Buy milk", "completed": false }` -->

## Business Rules

<!-- List any validation rules, constraints, or edge-case behaviour that the
     app should enforce. Remove this section if there are none.

     Example:
     - Item titles must not be empty or whitespace-only.
     - Duplicate titles are allowed.
     - Deleting the last item resets the filter to "All". -->
