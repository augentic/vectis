## 1. Build Infrastructure

- [ ] 1.1 Run cargo xcode in the app directory to generate shared.xcodeproj
- [ ] 1.2 Create the target directory for the iOS shell

## 2. Generate iOS Shell

- [ ] 2.1 Invoke ios-writer skill (read .cursor/skills/ios-writer/SKILL.md, follow Create Mode)

## 3. Build Verification

- [ ] 3.1 Run make setup in the iOS directory
- [ ] 3.2 Run make build in the iOS directory
- [ ] 3.3 Run swiftformat --lint on the generated Swift files

## 4. Code Review

- [ ] 4.1 Invoke the ios-reviewer skill with the app directory
      (Read .cursor/skills/ios-reviewer/SKILL.md and follow its process)
- [ ] 4.2 Address all Critical and Warning findings
- [ ] 4.3 Re-run make build and swiftformat --lint after fixes
