use sweet::{
    token::{Key, KeyAttribute, Modifier},
    Binding, Definition, ParseError, SwhkdParser,
};

#[test]
fn test_basic_keybind() -> Result<(), ParseError> {
    let contents = "
r
    alacritty
            ";
    SwhkdParser::from(&contents)?;
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
    let parsed = SwhkdParser::from(&contents)?;

    let known = vec![
        Binding {
            definition: Definition {
                modifiers: vec![],
                key: Key::new("r", KeyAttribute::None),
            },
            command: "alacritty".to_string().to_string(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![],
                key: Key::new("w", KeyAttribute::None),
            },
            command: "kitty".to_string().to_string(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![],
                key: Key::new("t", KeyAttribute::None),
            },
            command: "/bin/firefox".to_string().to_string(),
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
    let parsed = SwhkdParser::from(&contents)?;

    let known = vec![
        Binding {
            definition: Definition {
                modifiers: vec![],
                key: Key::new("r", KeyAttribute::None),
            },
            command: "alacritty".to_string().to_string(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![],
                key: Key::new("w", KeyAttribute::None),
            },
            command: "kitty".to_string().to_string(),
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

    let parsed = SwhkdParser::from(&contents)?;
    let known = vec![Binding {
        definition: Definition {
            modifiers: vec![Modifier("super".to_string())],
            key: Key::new("5", KeyAttribute::None),
        },
        command: "alacritty".to_string().to_string(),
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

    assert!(SwhkdParser::from(&contents).is_err());
}

#[test]
fn test_modifier_instead_of_keysym() {
    let contents = "
shift + k + alt
    notify-send 'Hello world!'
            ";

    assert!(SwhkdParser::from(&contents).is_err());
}

#[test]
fn test_unfinished_plus_sign() {
    let contents = "


shift + alt +
    notify-send 'Hello world!'
            ";
    assert!(SwhkdParser::from(&contents).is_err());
}

#[test]
fn test_plus_sign_at_start() {
    let contents = "
+ shift + k
    notify-send 'Hello world!'
            ";

    assert!(SwhkdParser::from(&contents).is_err());
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
    let parsed = SwhkdParser::from(&contents)?;

    let command = "notify-send 'Hello world!'".to_string();
    let known = vec![
        Binding {
            definition: Definition {
                modifiers: vec![Modifier("shift".to_string())],
                key: Key::new("k", KeyAttribute::None),
            },
            command: command.clone(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier("control".to_string())],
                key: Key::new("5", KeyAttribute::None),
            },
            command: command.clone(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier("alt".to_string())],
                key: Key::new("2", KeyAttribute::None),
            },
            command: command.clone(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier("altgr".to_string())],
                key: Key::new("i", KeyAttribute::None),
            },
            command: command.clone(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![Modifier("super".to_string())],
                key: Key::new("z", KeyAttribute::None),
            },
            command: command.clone(),
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
    let parsed = SwhkdParser::from(&contents)?;

    let known = vec![Binding {
        definition: Definition {
            modifiers: vec![],
            key: Key::new("p", KeyAttribute::None),
        },
        command: String::from("xbacklight -inc 10 -fps 30 -time 200"),
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

    assert!(SwhkdParser::from(&contents).is_err());
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

    assert!(SwhkdParser::from(&contents).is_err());
}
