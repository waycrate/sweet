# `sweet`
Simple Wayland Event Encoding Text

V2 config file parser for [swhkd](https://github.com/waycrate/swhkd.git).
Developed through Google Summer of Code 2024.

### What it does

This sweet little parser is to be gradually integrated into the swhkd repo itself.
For now, the grammar for the config parser is being implemented in this isolated
repository.

As of now, the parser prints all the bindings that a given inputs file can expand to.

### Progress

Here's what has been worked on so far.

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
- [x] Comments
- [ ] Tests (need to port them into proper cargo tests)
- [ ] Integration into upstream

Want to learn how the code works? Check out [my blog](https://lavafroth.is-a.dev/tags/google-summer-of-code/) where I cover each topic as I implement them.

### How to test it out?

All the examples that are currently tested against are piled up in the
`hotkeys.swhkd` file. These are to be ported to actual cargo tests.

Feel free to look through the file and uncomment some of the erroneous
examples to try them out!

```
cargo r -- hotkeys.swhkd
```

### Contributing

This repo is for cathedral style development. I'm assigned with creating the parser and there's potential financial transactions involved. Contributions are NOT welcome.
