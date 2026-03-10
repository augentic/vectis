## App Concept

A to-do list manager that synchronises items between a REST API server and local
storage. The app works fully offline -- queuing mutations and flushing them when
connectivity is available -- and receives real-time push updates from the server
so changes made by other clients appear automatically.

## Motivation

Replicate the existing `examples/todo` application using the OpenSpec-driven
workflow to validate that the `crux-app` schema + `core-writer` skill pipeline
produces an equivalent Crux shared crate. This serves as the reference comparison
for the spec-driven generation approach.

## Target Directory

`examples/opsx_todo`

## Capabilities Overview

- **HTTP** -- CRUD operations against a REST API (`/api/todos`) for creating,
  reading, updating, and deleting to-do items.
- **Key-Value storage** -- Persists the item list and pending operation queue
  locally so the app survives restarts and works offline.
- **Timer / Time** -- Periodic retry timer (30 seconds) to flush pending
  operations when previous sync attempts failed.
- **Server-Sent Events** -- Subscribes to a push stream from the server for
  real-time create/update/delete notifications from other clients.
- **Platform detection** -- Not needed.
