## 1. Project Setup

- [x] 1.1 Create the target directory `examples/opsx_todo`
- [x] 1.2 Copy app-spec.md to `examples/opsx_todo/spec.md`

## 2. Generate Crux Core

- [x] 2.1 Invoke the core-writer skill with `spec-file=examples/opsx_todo/spec.md` and `project-dir=examples/opsx_todo` (read `.cursor/skills/core-writer/SKILL.md` and follow its Create Mode process)

## 3. Verification

- [x] 3.1 Run `cargo check` in the project directory
- [x] 3.2 Run `cargo test` in the project directory
- [x] 3.3 Run `cargo clippy --all-targets` in the project directory

## 4. Code Review

- [x] 4.1 Invoke the core-reviewer skill with `target-dir=examples/opsx_todo` (read `.cursor/skills/core-reviewer/SKILL.md` and follow its process)
- [x] 4.2 Address all Critical and Warning findings
- [x] 4.3 Re-run `cargo check`, `cargo test`, `cargo clippy` after fixes
