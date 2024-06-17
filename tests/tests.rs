use sweet::{
    token::{Key, KeyAttribute},
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
                key: Key {
                    key: "r".to_string(),
                    attribute: KeyAttribute::None,
                },
            },
            command: "alacritty".to_string().to_string(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![],
                key: Key {
                    key: "w".to_string(),
                    attribute: KeyAttribute::None,
                },
            },
            command: "kitty".to_string().to_string(),
        },
        Binding {
            definition: Definition {
                modifiers: vec![],
                key: Key {
                    key: "t".to_string(),
                    attribute: KeyAttribute::None,
                },
            },
            command: "/bin/firefox".to_string().to_string(),
        },
    ];

    assert_eq!(parsed.bindings, known);

    Ok(())
}
