## App Reference

`examples/opsx_todo` — a Crux todo-list app with offline-first sync, SSE
real-time updates, and HTTP CRUD. The shared crate is fully generated and
passes all checks (31 tests, zero clippy warnings).

## Target Directory

`examples/opsx_todo/iOS`

## Motivation

Learning exercise and end-to-end validation of the Crux → iOS shell pipeline.
The core already exercises HTTP, Key-Value, Time, and a custom SSE capability,
making it a good integration test for the ios-writer skill.

## Design System Notes

Use the default VectisDesign tokens. No overrides needed for this example app.
