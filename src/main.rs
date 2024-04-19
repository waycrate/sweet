use anyhow::{bail, Result};
use itertools::Itertools;
use std::{fmt::Debug, fs};

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template.pest"]
pub struct SwhkdParser;

#[derive(Debug)]
pub enum Token {
    Modifier(String),
    Key(String),
    Command(String),
}

trait ComponentToString {
    fn inner_string(&self) -> String;
}

impl ComponentToString for Pair<'_, Rule> {
    fn inner_string(&self) -> String {
        self.as_str().to_string()
    }
}

fn extract_trigger(component: Pair<'_, Rule>) -> Vec<Token> {
    match component.as_rule() {
        Rule::modifier => vec![Token::Modifier(component.inner_string())],
        Rule::modifier_shorthand | Rule::modifier_omit_shorthand => component
            .into_inner()
            .map(|component| Token::Modifier(component.inner_string()))
            .collect(),
        Rule::shorthand => {
            let mut keys = vec![];
            for shorthand_component in component.into_inner() {
                match shorthand_component.as_rule() {
                    Rule::keybind => keys.push(Token::Key(shorthand_component.inner_string())),
                    Rule::key_range => {
                        let (lower_bound, upper_bound) = extract_bounds(shorthand_component);
                        keys.extend(
                            (lower_bound..=upper_bound).map(|key| Token::Key(key.to_string())),
                        );
                    }
                    _ => {}
                }
            }
            keys
        }
        Rule::keybind => vec![Token::Key(component.inner_string())],
        _ => vec![],
    }
}

fn unbind_parser(pair: Pair<'_, Rule>) {
    let mut unbind = vec![];
    for component in pair.into_inner() {
        unbind.push(extract_trigger(component));
    }
    let unbind_cartesian_product: Vec<_> = unbind.iter().multi_cartesian_product().collect();
    for trigger_to_unbind in unbind_cartesian_product {
        println!("unbind: {:?}", trigger_to_unbind);
    }
}

fn import_parser(pair: Pair<'_, Rule>) {
    let mut imports = vec![];
    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::import_file => imports.push(component.inner_string()),
            _ => unreachable!(),
        }
    }

    for import in imports {
        println!("import: {}", import);
    }
}

fn extract_bounds(pair: Pair<'_, Rule>) -> (char, char) {
    let mut bounds = pair.into_inner();
    let lower_bound: char = bounds
        .next()
        .unwrap()
        .as_str()
        .parse()
        .expect("failed to parse lower bound");
    let upper_bound: char = bounds
        .next()
        .unwrap()
        .as_str()
        .parse()
        .expect("failed to parse upper bound");

    if !lower_bound.is_ascii() || !upper_bound.is_ascii() {
        panic!("lower and upper bounds are not ascii");
    }
    assert!(lower_bound < upper_bound);
    (lower_bound, upper_bound)
}

fn parse_command_shorthand(pair: Pair<'_, Rule>) -> Vec<Token> {
    let mut command_variants = vec![];

    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::command_component => {
                command_variants.push(Token::Command(component.inner_string()))
            }
            Rule::range => {
                let (lower_bound, upper_bound) = extract_bounds(component);
                command_variants
                    .extend((lower_bound..=upper_bound).map(|key| Token::Command(key.to_string())));
            }
            _ => {}
        }
    }
    command_variants
}

fn binding_parser(pair: Pair<'_, Rule>) {
    let mut tokens = vec![];
    let mut comm = vec![];
    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::command => {
                for subcomponent in component.into_inner() {
                    match subcomponent.as_rule() {
                        Rule::command_component => {
                            comm.push(vec![Token::Command(subcomponent.inner_string())]);
                        }
                        Rule::command_with_brace => {
                            comm.push(parse_command_shorthand(subcomponent));
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                let trigger = extract_trigger(component);
                if !trigger.is_empty() {
                    tokens.push(trigger);
                }
            }
        }
    }
    let bind_cartesian_product: Vec<_> = tokens.iter().multi_cartesian_product().collect();
    let command_cartesian_product: Vec<_> = comm.iter().multi_cartesian_product().collect();
    let bind_len = bind_cartesian_product.len();
    let command_len = command_cartesian_product.len();

    assert_eq!(
        bind_len, command_len,
        "the cartesian products of the binding variants {bind_len} does not equal the possible command variants {command_len}."
    );

    let composition: Vec<_> = bind_cartesian_product
        .into_iter()
        .zip(command_cartesian_product.into_iter())
        .collect();

    for (binding, command) in composition {
        println!("{:?} => {:?}", binding, command);
    }
}

fn main() -> Result<()> {
    let Some(arg) = std::env::args().nth(1) else {
        bail!("please supply a path to a hotkeys config file");
    };
    let raw_content = fs::read_to_string(arg)?;
    let parse_result = SwhkdParser::parse(Rule::main, &raw_content)?;
    let Some(main) = parse_result.into_iter().next() else {
        bail!("a hotkeys config file must contain one and only one main section");
    };

    for decl in main.into_inner() {
        match decl.as_rule() {
            Rule::binding => binding_parser(decl),
            Rule::unbind => unbind_parser(decl),
            Rule::import => import_parser(decl),
            // End of identifier
            // Here, it means the end of the file.
            Rule::EOI => {}
            _ => unreachable!(),
        }
        println!("-----");
    }
    Ok(())
}
