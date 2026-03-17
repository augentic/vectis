## 1. Build Infrastructure

- [x] 1.1 Run `cargo xcode` in `examples/opsx_todo` to generate `shared.xcodeproj`
- [x] 1.2 Create the target directory `examples/opsx_todo/iOS`

## 2. Generate iOS Shell

- [x] 2.1 Invoke the ios-writer skill with `app-dir=examples/opsx_todo` and `project-dir=examples/opsx_todo/iOS` (read `.cursor/skills/ios-writer/SKILL.md` and follow its Create Mode process)

## 3. Build Verification

- [x] 3.1 Run `make setup` in `examples/opsx_todo/iOS`
- [x] 3.2 Run `make build` in `examples/opsx_todo/iOS`
- [x] 3.3 Run `swiftformat --lint` on the generated Swift files

## 4. Code Review

- [x] 4.1 Invoke the ios-reviewer skill with `target-dir=examples/opsx_todo` (read `.cursor/skills/ios-reviewer/SKILL.md` and follow its process)
- [x] 4.2 Address all Critical and Warning findings
- [x] 4.3 Re-run `make build` and `swiftformat --lint` after fixes
