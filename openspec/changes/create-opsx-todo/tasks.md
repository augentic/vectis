## 1. Project Setup

- [ ] 1.1 Create the target directory `examples/opsx_todo`
- [ ] 1.2 Copy `app-spec.md` from this change to `examples/opsx_todo/spec.md`

## 2. Generate Crux Core

- [ ] 2.1 Read the core-writer skill at `.cursor/skills/core-writer/SKILL.md` and invoke it in Create Mode with `spec-file=examples/opsx_todo/spec.md` and `project-dir=examples/opsx_todo`

## 3. Verification

- [ ] 3.1 Run `cargo check` in `examples/opsx_todo`
- [ ] 3.2 Run `cargo test` in `examples/opsx_todo`
- [ ] 3.3 Run `cargo clippy --all-targets` in `examples/opsx_todo`
