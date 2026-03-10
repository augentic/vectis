---
name: code-reviewer
description: Review generated Rust Crux application code for structural issues, logic bugs, and general quality problems. Use when the user wants to review, audit, or quality-check a Crux app, or after code generation to catch issues before archiving.
---

# Crux Code Reviewer

Systematically review a generated Crux application for structural issues,
logic bugs, and general code quality problems. Produces a severity-graded
report with actionable findings and suggested fixes.

This skill catches semantic issues that compilers, linters, and clippy miss:
missing `render()` calls, conflict-resolution gaps, pending-op coalescing bugs,
state machine incompleteness, and interaction-sequence race conditions.

## Arguments

| Argument | Required | Description |
|---|---|---|
| `target-dir` | **Yes** | Path to the Crux app directory to review (contains `shared/src/`) |
| `reference-dir` | No | Path to a known-good app for comparative review (e.g. `examples/todo`) |
| `scope` | No | `full` (default) runs all three passes; `quick` runs structural + quality only, skipping logic simulation |

## Process

### 1. Gather context

Read the following files from `{target-dir}`:

- `spec.md` -- the app specification (required for logic pass)
- `shared/Cargo.toml` -- dependencies and features
- All `.rs` files under `shared/src/` -- focus on `app.rs` (the `update()` function)

If `reference-dir` is provided, also read the corresponding files from the
reference app. Differences between the two highlight potential regressions.

### 2. Structural pass

Read `references/crux-review-checks.md` in this skill's directory.

Apply checks CRX-001 through CRX-010 against the source code. These are
pattern-based checks that scan for known Crux-specific issues:

- Missing `render()` after state mutations
- Missing serde derives on bridge-crossing types
- Input validation gaps on user-supplied text
- Timestamp completeness on `PendingOp` variants
- ViewModel field typing (typed values vs pre-formatted strings)
- Unused dependencies in `Cargo.toml`

For each violation found, record: check ID, file, line range, description,
severity (Critical or Warning), and suggested fix.

### 3. Logic pass (skip if scope = quick)

Read `references/logic-review-checks.md` in this skill's directory.

Apply checks LOG-001 through LOG-008. These require reasoning about event
sequences, not just pattern matching. For each check:

1. **LOG-001 State machine completeness** -- Enumerate every state enum
   (Page, SyncStatus, SseConnectionState, etc.). For each transition in
   `update()`, verify that all required side-effects fire (render, save, sync).
   Draw the state machine mentally; flag incomplete edges.

2. **LOG-002 Operation coalescing** -- Trace the sequence: Create -> Delete
   before sync. Does the code skip the server call for items that were never
   synced? Check both `DeleteTodo` and `ClearCompleted` handlers.

3. **LOG-003 Concurrent operation conflicts** -- Trace: sync in-flight +
   SSE event for the same item. Does `pending_ops.retain()` in the SSE handler
   clobber the in-flight sync state?

4. **LOG-004 Temporal ordering** -- For every conflict-resolution comparison,
   verify both sides have timestamps. Check `PendingOp` variants for missing
   temporal fields.

5. **LOG-005 Fallback-on-None** -- For every `unwrap_or_default()`, `Option`
   with `_ => true`, or `None` fallback, check if the default is semantically
   correct in the domain.

6. **LOG-006 Rapid-action sequences** -- Trace what happens when the user
   fires the same action twice before the first async operation completes.
   Check for duplicate pending ops or unbounded queue growth.

7. **LOG-007 Spec gap detection** -- Compare each user-facing Event variant
   against the Features section of `spec.md`. Flag events that accept untrusted
   input without validation that common sense requires (empty strings, negative
   numbers, duplicate IDs) even if the spec is silent.

8. **LOG-008 Missing edge-case tests** -- Cross-reference the `#[cfg(test)]`
   module against the interaction sequences from LOG-001--007. Each identified
   risk should have at least one test.

Record findings with severity Critical (data loss, incorrect server calls) or
Warning (stale UI, missing tests).

### 4. Quality pass

Read `references/general-review-checks.md` in this skill's directory.

Apply checks GEN-001 through GEN-012 against all `.rs` files. These are
language-level quality checks:

- No `unwrap()`/`expect()` in production code (tests exempt)
- No debug output (`println!`, `dbg!`, `eprintln!`)
- No hardcoded secrets or credentials
- Error propagation (not silent swallowing)
- Match arm exhaustiveness
- Serialization round-trip completeness
- Function length (under 50 lines)

Record findings with severity Warning or Info.

### 5. Comparative review (if reference-dir provided)

Compare structural decisions between the target and reference apps:

- Event variant signatures (do they carry timestamps/IDs from the shell?)
- PendingOp variant structure (do they carry enough data for conflict resolution?)
- ViewModel field types (typed vs pre-formatted)
- Test coverage breadth (count and categorize tests in both)

Flag significant divergences as Warning with a note explaining what the
reference app does differently and why.

### 6. Produce report

Output a structured report with the following format:

```
## Code Review Report: {app-name}

### Summary
- Critical: N findings
- Warning: N findings
- Info: N findings

### Critical Findings

#### [CRX-001] Missing render() after page transition
- **File**: shared/src/app.rs, lines 384-388
- **Issue**: Navigating from Error to Loading mutates `model.page` without
  emitting `render()`. The shell may not see the Loading state.
- **Fix**: Wrap the return in `render().and(Command::event(Event::Initialize))`

... (one block per finding, ordered by severity then file)

### Warning Findings
...

### Info Findings
...

### Test Gap Summary
- Missing test for: [scenario description]
- Missing test for: ...
```

### 7. Auto-fix mechanical issues

After presenting the report, offer to auto-fix findings that are mechanical:

- Adding missing `Serialize`/`Deserialize` derives
- Wrapping returns in `render().and(...)`
- Adding `.trim()` and empty checks on text inputs
- Removing unused dependencies from `Cargo.toml`

Do NOT auto-fix logic bugs (LOG-001 through LOG-008) without explicit
confirmation -- these require design decisions.

After any fixes, re-run `cargo check`, `cargo test`, and `cargo clippy` to
verify the fixes compile and pass.

## Severity Definitions

| Severity | Meaning | Action |
|---|---|---|
| **Critical** | Data loss, incorrect server calls, conflict-resolution failure, panic in production | Must fix before archive |
| **Warning** | Stale UI, missing tests, suboptimal types, unnecessary clones | Should fix; acceptable to defer with justification |
| **Info** | Style, documentation, minor improvements | Fix if convenient |

## Integration with OpenSpec Workflow

This skill is invoked as part of the `crux-app` schema's task list, after
code generation and compiler verification, before archive:

```
propose -> apply (core-writer) -> verify (cargo check/test/clippy) -> review (this skill) -> fix -> archive
```

The tasks artifact for a crux-app change includes a Code Review section that
invokes this skill. See the `crux-app` schema for details.

The skill can also be invoked standalone at any time:

> Use the code-reviewer skill to review `examples/my-app`

> Review `examples/my-app` against `examples/todo` as a reference
