## App Concept

A to-do list manager that synchronises items between a REST API server and local storage. The app works fully offline -- queuing mutations and flushing them when connectivity is available -- and receives real-time push updates from the server so changes made by other clients appear automatically.

## Motivation

Learning exercise and reference implementation demonstrating core Crux patterns: offline-first data management, optimistic UI updates, background sync with retry, and real-time server-sent event integration.

## Target Directory

examples/opsx_todo

## Capabilities Overview

- **HTTP** -- CRUD operations against a REST API for creating, reading, updating, and deleting todo items.
- **Key-Value storage** -- Persists the item list and pending operation queue locally so the app survives restarts and works offline.
- **Timer / Time** -- Periodic retry timer (30 seconds) to flush the pending operation queue when previous sync attempts failed. Also provides `now()` for `updated_at` timestamps.
- **Server-Sent Events** -- Subscribes to a push stream from the server to receive create/update/delete notifications from other clients in real time.
- **Platform detection** -- Not needed.
