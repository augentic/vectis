## 1. Build Infrastructure

- [ ] 1.1 Run `cargo xcode` in `examples/opsx_todo/` to generate `shared/shared.xcodeproj`
- [ ] 1.2 Create the target directory `examples/opsx_todo/iOS/`

## 2. Generate iOS Shell

- [ ] 2.1 Invoke the ios-writer skill with `app-dir=examples/opsx_todo` and `project-dir=examples/opsx_todo/iOS` (Read `.cursor/skills/ios-writer/SKILL.md` and follow its Create Mode process)

## 3. Build Verification

- [ ] 3.1 Run `make setup` in `examples/opsx_todo/iOS/` to generate the Xcode project from `project.yml`
- [ ] 3.2 Run `make build` in `examples/opsx_todo/iOS/` to compile for the simulator
- [ ] 3.3 Run `swiftformat --lint` on the generated Swift files in `examples/opsx_todo/iOS/`

## 4. Code Review

- [ ] 4.1 Invoke the ios-reviewer skill with `target-dir=examples/opsx_todo` (Read `.cursor/skills/ios-reviewer/SKILL.md` and follow its process)
- [ ] 4.2 Address all Critical and Warning findings from the review
- [ ] 4.3 Re-run `make build` and `swiftformat --lint` after fixes
