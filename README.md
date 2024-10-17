# `sweet`
Simple Wayland Event Encoding Text

V2 config file parser for [swhkd](https://github.com/waycrate/swhkd.git).
Developed through Google Summer of Code 2024.

### What it does

This sweet little parser is to be gradually integrated into the swhkd repo itself.
For now, the grammar for the config parser is being implemented in this isolated
repository.

As of now, the parser prints all the bindings that a given inputs file can expand to.

### Roadmap

- [x] Bindings
  - [x] Modifiers
  - [x] Regular keys
  - [x] Shorthands
  - [x] Ranges
  - [x] Omissions
  - [x] _Send_ and _on release_ attributes
- [x] Unbinds
- [x] Modes
  - [x] Oneoff
  - [x] Swallow
  - [x] `@mode` in commands
- [x] Comments
- [x] Imports
  - [x] Merge definitions from all imports
- [x] Tests
- [x] Integration into [downstream](https://github.com/waycrate/swhkd)

## Extra features
- [x] Warn user if input config is not a regular file
- [x] Set a maximum file size cap for configs (limit configurable in the `build.rs`)
- [x] Map keys and modifiers to internal representation (evdev enum variants) in a single pass


Want to learn how the code works? Check out [my blog](https://lavafroth.is-a.dev/tags/google-summer-of-code/) where I cover each topic as I implement them.

### See it in action

To see a structured representation of the sample config file after parsing, run the following:

```
cargo r -- hotkeys.swhkd
```

To run all available tests, run `cargo test`
