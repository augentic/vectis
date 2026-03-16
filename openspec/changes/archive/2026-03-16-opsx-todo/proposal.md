## App Concept

A to-do list manager that synchronises items between a REST API server and local
storage. The app works fully offline -- queuing mutations and flushing them when
connectivity is available -- and receives real-time push updates from the server
so changes made by other clients appear automatically.

## Motivation

Learning exercise and reference implementation demonstrating the Crux framework's
capability model: HTTP for API calls, Key-Value storage for offline persistence,
Server-Sent Events for real-time sync, and timers for retry logic.

## Target Directory

examples/opsx_todo

## Capabilities Overview

- **HTTP** -- CRUD operations against a REST API for creating, reading, updating, and deleting todo items.
- **Key-Value storage** -- Persists the item list and pending operation queue locally so the app survives restarts and works offline.
- **Timer / Time** -- Periodic retry timer (every 30 seconds) to flush the pending queue when previous sync attempts fail. Also provides `now()` for `updated_at` timestamps.
- **Server-Sent Events** -- Subscribes to a push stream from the server to receive create/update/delete notifications from other clients in real time.
- **Platform detection** -- Not needed.
