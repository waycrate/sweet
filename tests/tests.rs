use std::{collections::BTreeSet, io::Write};

use pest::error::LineColLocation::{Pos, Span};
use sweet::{
    token::{Key, KeyAttribute, Modifier},
    Binding, Definition, ParseError, ParserInput, SwhkdParser,
};
use thiserror::Error;

fn assert_grammar_error_at(contents: &str, pos: (usize, usize)) {
    let parse_result = SwhkdParser::from(ParserInput::Raw(contents));
    let Err(ParseError::Grammar(e)) = parse_result else {
        panic!("expected grammar parse error")
    };
    assert_eq!(e.line_col, Pos(pos));
}
fn assert_grammar_error_at_span(contents: &str, start: (usize, usize), end: (usize, usize)) {
    let parse_result = SwhkdParser::from(ParserInput::Raw(contents));
    let Err(ParseError::Grammar(e)) = parse_result else {
        panic!("expected grammar parse error")
    };
    assert_eq!(e.line_col, Span(start, end));
}

fn assert_equal_binding_set(a: Vec<Binding>, mut b: Vec<Binding>) {
    for binding in a {
        if let Some(pos) = b.iter().position(|bin| bin.eq(&binding)) {
            b.remove(pos);
        }
    }
    assert_eq!(b, vec![]);
}

#[test]
fn test_existing_file() -> std::io::Result<()> {
    let mut setup = tempfile::NamedTempFile::new()?;
    let setup_path = setup.path().to_owned();
    setup.write_all(
        b"
x
    dmenu_run

q
    bspc node -q",
    )?;
    // setup gets dropped here
    std::fs::read_to_string(setup_path)?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum IoOrParseError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[test]
fn test_load_multiple_config() -> Result<(), IoOrParseError> {
    let mut setup2 = tempfile::NamedTempFile::new()?;
    setup2.write_all(
        b"
super + c
    hello",
    )?;

    let mut setup = tempfile::NamedTempFile::new()?;
    write!(
        setup,
        "
include {}
super + b
   firefox",
        setup2.path().display()
    )?;

    let parsed = SwhkdParser::from(ParserInput::Path(setup.path()))?;
    let known = [
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_B, KeyAttribute::None),
            },
            command: "firefox".to_string(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_C, KeyAttribute::None),
            },
            command: "hello".to_string(),
            mode_instructions: vec![],
        },
    ];
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_circular_import() -> Result<(), IoOrParseError> {
    let mut setup = tempfile::NamedTempFile::new()?;
    setup.write_all(
        b"
a
    a",
    )?;

    let mut setup2 = tempfile::NamedTempFile::new()?;
    write!(
        setup2,
        "
include {}
b
    b",
        setup.path().display()
    )?;
    let mut setup3 = tempfile::NamedTempFile::new()?;
    let setup3_path = setup3.path().to_owned();
    let mut setup4 = tempfile::NamedTempFile::new()?;
    write!(
        setup3,
        "
include {}
include {}
include {}
include {}
c
    c",
        setup.path().display(),
        setup2.path().display(),
        setup3_path.display(),
        setup4.path().display(),
    )?;
    write!(
        setup4,
        "
include {}
d
    d",
        setup3.path().display()
    )?;
    let parsed = SwhkdParser::from(ParserInput::Path(setup4.path()))?;
    let known = vec![
        Binding {
            definition: Definition::new_simple_key(evdev::Key::KEY_D),
            command: "d".to_string(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition::new_simple_key(evdev::Key::KEY_C),
            command: "c".to_string(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition::new_simple_key(evdev::Key::KEY_B),
            command: "b".to_string(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition::new_simple_key(evdev::Key::KEY_A),
            command: "a".to_string(),
            mode_instructions: vec![],
        },
    ];
    assert_equal_binding_set(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_include_and_unbind() -> Result<(), IoOrParseError> {
    let mut setup2 = tempfile::NamedTempFile::new()?;
    setup2.write_all(
        b"
super + c
    hello
super + d
    world",
    )?;

    let mut setup = tempfile::NamedTempFile::new()?;
    write!(
        setup,
        "
include {}
super + b
   firefox
ignore super + d",
        setup2.path().display()
    )?;

    let parsed = SwhkdParser::from(ParserInput::Path(setup.path()))?;
    let known = [
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_B, KeyAttribute::None),
            },
            command: "firefox".to_string(),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_C, KeyAttribute::None),
            },
            command: "hello".to_string(),
            mode_instructions: vec![],
        },
    ];
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_basic_keybind() -> Result<(), ParseError> {
    let contents = "
r
    alacritty
            ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    let known = [Binding {
        definition: Definition::new_simple_key(evdev::Key::KEY_R),
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
// this test is stricter than the previous parser
// gimp is not a valid key or a modifier
fn test_commented_out_keybind() {
    let contents = "
#w
    gimp";
    assert_grammar_error_at(&contents, (3, 6));
}

#[test]
fn test_blank_config() -> Result<(), ParseError> {
    let contents = "";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, vec![]);
    Ok(())
}

#[test]
fn test_blank_config_with_whitespace() -> Result<(), ParseError> {
    let contents = "


            ";
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, vec![]);
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
fn test_range_syntax_ascii_character() -> Result<(), ParseError> {
    let contents = "
super + {a-c}
    {firefox, brave, librewolf}
    ";
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_A, KeyAttribute::None),
            },
            command: String::from("firefox"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_B, KeyAttribute::None),
            },
            command: String::from("brave"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_C, KeyAttribute::None),
            },
            command: String::from("librewolf"),
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
fn test_valid_curly_brace_config() -> Result<(), ParseError> {
    let contents = "
super + {a,b,c}
    {firefox, brave, librewolf}
    ";
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_A, KeyAttribute::None),
            },
            command: String::from("firefox"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_B, KeyAttribute::None),
            },
            command: String::from("brave"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_C, KeyAttribute::None),
            },
            command: String::from("librewolf"),
            mode_instructions: vec![],
        },
    ];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_valid_bspwm_curly_brace_config() -> Result<(), ParseError> {
    let contents = "
super + {h,j,k,l}
    bspc node -p {west, south, north, east}
    ";
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_H, KeyAttribute::None),
            },
            command: String::from("bspc node -p west"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_J, KeyAttribute::None),
            },
            command: String::from("bspc node -p south"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_K, KeyAttribute::None),
            },
            command: String::from("bspc node -p north"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_L, KeyAttribute::None),
            },
            command: String::from("bspc node -p east"),
            mode_instructions: vec![],
        },
    ];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_invalid_curly_brace_config() -> Result<(), ParseError> {
    let contents = "
super + }a,b,c{
    {firefox, brave, librewolf}
    ";
    assert_grammar_error_at(contents, (2, 9));
    Ok(())
}

#[test]
// Stricter than previous parser.
fn test_curly_brace_less_commands() -> Result<(), ParseError> {
    let contents = "
super + {a,b,c}
    {firefox, brave}
    ";
    assert_grammar_error_at_span(contents, (2, 1), (3, 21));
    Ok(())
}

#[test]
// Stricter than previous parser.
fn test_curly_brace_less_keysyms() -> Result<(), ParseError> {
    let contents = "
super + {a,b}
    {firefox, brave, librewolf}
    ";
    assert_grammar_error_at_span(contents, (2, 1), (3, 32));
    Ok(())
}

#[test]
// Single variant curly brace shorthands are disallowed.
// We must have at least two variants for shorthands to make sense.
fn test_curly_brace_single_variant() -> Result<(), ParseError> {
    let contents = "
super + {a}
    {firefox}
    ";
    assert_grammar_error_at(contents, (2, 10));
    Ok(())
}

#[test]
fn test_omission() -> Result<(), ParseError> {
    let contents = "
super + {_, shift +} b
    {firefox, brave}";
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_B, KeyAttribute::None),
            },
            command: String::from("firefox"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super, Modifier::Shift].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_B, KeyAttribute::None),
            },
            command: String::from("brave"),
            mode_instructions: vec![],
        },
    ];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
}

#[test]
fn test_range_syntax() -> Result<(), ParseError> {
    let contents = "
super + {1-9,0}
    bspc desktop -f '{1-9,0}'";

    let known = vec![
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_1, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '1'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_2, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '2'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_3, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '3'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_4, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '4'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_5, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '5'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_6, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '6'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_7, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '7'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_8, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '8'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_9, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '9'"),
            mode_instructions: vec![],
        },
        Binding {
            definition: Definition {
                modifiers: [Modifier::Super].into_iter().collect(),
                key: Key::new(evdev::Key::KEY_0, KeyAttribute::None),
            },
            command: String::from("bspc desktop -f '0'"),
            mode_instructions: vec![],
        },
    ];
    let parsed = SwhkdParser::from(ParserInput::Raw(&contents))?;
    assert_eq!(parsed.bindings, known);
    Ok(())
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
