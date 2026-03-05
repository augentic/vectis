# Vectis

Toolkit for building applications with a user interface.

## Project Goals

- Support as many runtime platforms as possible, focusing on Web Browser, iOS and Android devices without excluding Windows, MacOS or Linux desktops.
- Contain all of the application behaviour in a shared core that can be tested independently of the runtime platform.
- Has a very opinionated application structure that makes it easier for AI code generation to get right.

## CRUX

The project goals are also shared by the [CRUX](https://github.com/redbadger/crux) framework and is written in Rust, the portable, fast and safe programming language favoured by our Augentic frameworks. So this toolkit targets CRUX code generation for the core of applications.

Familiarize yourself with how CRUX works by scanning the [documentation](https://docs.rs/crux_core/latest/crux_core/)

## Developer Setup

- [Install Rust](https://rust-lang.org/tools/install/)
- [Install Cursor](https://cursor.com/home)
- Install the (Rust Analyzer)[https://open-vsx.org/extension/rust-lang/rust-analyzer] Cursor extension

### iOS/MacOS Development

[Install Xcode command line tools](https://developer.apple.com/documentation/xcode/installing-the-command-line-tools/)

```shell
# Builder for Swift projects without needing Xcode UI
brew install xcode-build-server

# Pretty print formatter for `xcodebuild` command output in Cursor terminal
brew install xcbeautify

# Allow for advanced formatting and language features
brew install swiftformat
```

Install the [Swift Language Support](https://open-vsx.org/extension/chrisatwindsurf/swift-vscode)
Install the [SweetPad](https://marketplace.visualstudio.com/items?itemName=SweetPad.sweetpad) Cursor extension to link Cursor to Xcode.