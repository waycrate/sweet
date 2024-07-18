use std::collections::BTreeSet;

use pest::error::LineColLocation::Pos;
use sweet::{
    token::{Key, KeyAttribute, Modifier},
    Binding, Definition, ParseError, ParserInput, SwhkdParser,
};

fn assert_grammar_error_at(contents: &str, pos: (usize, usize)) {
    let parse_result = SwhkdParser::from(ParserInput::Raw(contents));
    let Err(ParseError::Grammar(e)) = parse_result else {
        panic!("expected grammar parse error")
    };
    assert_eq!(e.line_col, Pos(pos));
}

#[test]
fn test_basic_keybind() -> Result<(), ParseError> {
    let contents = "
r
    alacritty
            ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    let known = [Binding {
        definition: Definition {
            modifiers: BTreeSet::default(),
            key: Key::new(evdev::Key::KEY_R, KeyAttribute::None),
        },
        command: "alacritty".to_string(),
        mode_instructions: vec![],
    }];

    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_multiple_keybinds() -> Result<(), ParseError> {
    let contents = "
r
    alacritty

w
    kitty

t
    /bin/firefox
        ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;

    let known = vec![
        Binding {
            definition: Definition {
                modifiers: BTreeSet::default(),
                key: Key::new(evdev::Key::KEY_R, KeyAttribute::None),
            },
            command: "alacritty".to_string(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: BTreeSet::default(),
                key: Key::new(evdev::Key::KEY_W, KeyAttribute::None),
            },
            command: "kitty".to_string(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: BTreeSet::default(),
                key: Key::new(evdev::Key::KEY_T, KeyAttribute::None),
            },
            command: "/bin/firefox".to_string(),
            mode_instructions: vec![],
        },
    ];

    assert_eq!(parsed.bindings, known);

    Ok(())
}

#[test]
fn test_comments() -> Result<(), ParseError> {
    let contents = "
r
    alacritty

w
    kitty

#t
    #/bin/firefox
        ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;

    let known = vec![
        Binding {
            definition: Definition {
                modifiers: BTreeSet::default(),
                key: Key::new(evdev::Key::KEY_R, KeyAttribute::None),
            },
            command: "alacritty".to_string(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: BTreeSet::default(),
                key: Key::new(evdev::Key::KEY_W, KeyAttribute::None),
            },
            command: "kitty".to_string(),
            mode_instructions: vec![],
        },
    ];

    assert_eq!(parsed.bindings, known);

    Ok(())
}

#[test]
fn test_multiple_keypress() -> Result<(), ParseError> {
    let contents = "
super + 5
    alacritty
        ";

    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    let known = vec![Binding {
        definition: Definition {
            modifiers: vec![Modifier::Super].into_iter().collect(),
            key: Key::new(evdev::Key::KEY_5, KeyAttribute::None),
        },
        command: "alacritty".to_string(),
        mode_instructions: vec![],
    }];

    assert_eq!(parsed.bindings, known);

    Ok(())
}

#[test]
fn test_keysym_instead_of_modifier() {
    let contents = "
shift + k + m
    notify-send 'Hello world!'
            ";

    let parse_result = SwhkdParser::from(ParserInput::Raw(&contents));
    let Err(ParseError::Grammar(e)) = parse_result else {
        panic!("expected grammar parse error")
    };
    assert_eq!(e.line_col, Pos((2, 11)));
}

#[test]
fn test_modifier_instead_of_keysym() {
    let contents = "
shift + k + alt
    notify-send 'Hello world!'
            ";
    assert_grammar_error_at(&contents, (2, 11));
}

#[test]
fn test_unfinished_plus_sign() {
    let contents = "


shift + alt +
    notify-send 'Hello world!'
            ";
    assert_grammar_error_at(&contents, (4, 14));
}

#[test]
fn test_plus_sign_at_start() {
    let contents = "
+ shift + k
    notify-send 'Hello world!'
            ";
    assert_grammar_error_at(&contents, (2, 1));
}

#[test]
fn test_common_modifiers() -> Result<(), ParseError> {
    let contents = "
shift + k
    notify-send 'Hello world!'

control + 5
    notify-send 'Hello world!'

alt + 2
    notify-send 'Hello world!'

altgr + i
    notify-send 'Hello world!'

super + z
    notify-send 'Hello world!'
            ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;

    let command = "notify-send 'Hello world!'".to_string();
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Shift].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_K, KeyAttribute::None),
            },
            command: command.clone(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Control].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_5, KeyAttribute::None),
            },
            command: command.clone(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Alt].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_2, KeyAttribute::None),
            },
            command: command.clone(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Altgr].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_I, KeyAttribute::None),
            },
            command: command.clone(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_Z, KeyAttribute::None),
            },
            command: command.clone(),
            mode_instructions: vec![],
        },
    ];

    assert_eq!(parsed.bindings, known);

    Ok(())
}

#[test]
fn test_command_with_many_spaces() -> Result<(), ParseError> {
    let contents = "
p
    xbacklight -inc 10 -fps 30 -time 200
        ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;

    let known = vec![Binding {
        definition: Definition {
            modifiers: BTreeSet::default(),
            key: Key::new(evdev::Key::KEY_P, KeyAttribute::None),
        },
        command: String::from("xbacklight -inc 10 -fps 30 -time 200"),
        mode_instructions: vec![],
    }];

    assert_eq!(parsed.bindings, known);

    Ok(())
}

#[test]
fn test_invalid_keybinding() {
    let contents = "
p
    xbacklight -inc 10 -fps 30 -time 200

pesto
    xterm
                    ";
    assert_grammar_error_at(&contents, (5, 2));
}

#[test]
// NOTE: This behavior is stricter than the older parser.
// Don't silently ignore keysyms not followed by command.
fn test_no_command() {
    let contents = "
k
    xbacklight -inc 10 -fps 30 -time 200

w

                    ";

    assert!(SwhkdParser::from(ParserInput::Raw(&contents)).is_err());
}

#[test]
fn test_real_config_snippet() -> Result<(), ParseError> {
    let contents = "
# reloads sxhkd configuration:
super + Escape
    pkill -USR1 -x sxhkd ; sxhkd &

# Launch Terminal
super + Return
    alacritty -t \"Terminal\" -e \"$HOME/.config/sxhkd/new_tmux_terminal.sh\"

# terminal emulator (no tmux)
super + shift + Return
    alacritty -t \"Terminal\"

# terminal emulator (new tmux session)
alt + Return
    alacritty -t \"Terminal\" -e \"tmux\"

ctrl + 0
    play-song.sh

super + minus
    play-song.sh album
                    ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;

    let known = vec![
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_ESC, KeyAttribute::None),
            },
            command: String::from("pkill -USR1 -x sxhkd ; sxhkd &"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_ENTER, KeyAttribute::None),
            },
            command: String::from(
                "alacritty -t \"Terminal\" -e \"$HOME/.config/sxhkd/new_tmux_terminal.sh\"",
            ),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super, Modifier::Shift].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_ENTER, KeyAttribute::None),
            },
            command: String::from("alacritty -t \"Terminal\""),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Alt].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_ENTER, KeyAttribute::None),
            },
            command: String::from("alacritty -t \"Terminal\" -e \"tmux\""),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Control].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_0, KeyAttribute::None),
            },
            command: String::from("play-song.sh"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_MINUS, KeyAttribute::None),
            },
            command: String::from("play-song.sh album"),
            mode_instructions: vec![],
        },
    ];

    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_multiline_command() -> Result<(), ParseError> {
    let contents = "
k
    mpc ls | dmenu | \\
    sed -i 's/foo/bar/g'
                    ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;

    let known = vec![Binding {
        definition: Definition {
            modifiers: BTreeSet::default(),
            key: Key::new(evdev::Key::KEY_K, KeyAttribute::None),
        },
        command: String::from("mpc ls | dmenu | sed -i 's/foo/bar/g'"),
        mode_instructions: vec![],
    }];

    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_case_insensitive() -> Result<(), ParseError> {
    let contents = "
Super + SHIFT + alt + a
    st
ReTurn
    ts
            ";
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Super, Modifier::Shift, Modifier::Alt]
                    .into_iter()
                    .collect(),
                key: Key::new(evdev::Key::KEY_A, KeyAttribute::None),
            },
            command: String::from("st"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: BTreeSet::default(),
                key: Key::new(evdev::Key::KEY_ENTER, KeyAttribute::None),
            },
            command: String::from("ts"),
            mode_instructions: vec![],
        },
    ];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_override() -> Result<(), ParseError> {
    let contents = "
super + a
    1
super + a
    2";
    let known = vec![Binding {
        definition: Definition {
            modifiers: vec![Modifier::Super].into_iter().collect(),
            key: Key::new(evdev::Key::KEY_A, KeyAttribute::None),
        },
        command: String::from("2"),
        mode_instructions: vec![],
    }];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_any_modifier() -> Result<(), ParseError> {
    let contents = "
any + a
    1";
    let known = vec![Binding {
        definition: Definition {
            modifiers: vec![Modifier::Any].into_iter().collect(),
            key: Key::new(evdev::Key::KEY_A, KeyAttribute::None),
        },
        command: String::from("1"),
        mode_instructions: vec![],
    }];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_duplicate_hotkeys() -> Result<(), ParseError> {
    let contents = "
super + shift + a
    st
shift + suPer +   A
    ts
b
    st
B
    ts
";
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: vec![Modifier::Super, Modifier::Shift].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_A, KeyAttribute::None),
            },
            command: String::from("ts"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: BTreeSet::default(),
                key: Key::new(evdev::Key::KEY_B, KeyAttribute::None),
            },
            command: String::from("ts"),
            mode_instructions: vec![],
        },
    ];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_range_syntax_not_ascii() {
    let contents = "
super + {a-是}
    {firefox, brave}
    ";
    assert_grammar_error_at(contents, (2, 12));
}

#[test]
fn test_range_syntax_invalid_range() {
    let contents = "
super + {bc-ad}
    {firefox, brave}
    ";
    assert_grammar_error_at(contents, (2, 10));
}

#[test]
fn test_ranger_syntax_not_full_range() {
    let contents = "
super + {a-}
    {firefox, brave}";
    assert_grammar_error_at(contents, (2, 12));
}

#[test]
fn test_period_escape_binding() -> Result<(), ParseError> {
    let contents = "
super + {\\,, .}
	riverctl focus-output {previous, next}";
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_COMMA, KeyAttribute::None),
            },
            command: String::from("riverctl focus-output previous"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_DOT, KeyAttribute::None),
            },
            command: String::from("riverctl focus-output next"),
            mode_instructions: vec![],
        },
    ];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
}
