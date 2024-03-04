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

fn binding_parser(pair: Pair<'_, Rule>) {
    let mut tokens = vec![];
    let mut comm = vec![];
    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::modifier => {
                tokens.push(vec![Token::Modifier(component.as_str().to_string())]);
            }

            Rule::modifier_range | Rule::modifier_omit_range => {
                tokens.push(
                    component
                        .into_inner()
                        .map(|component| Token::Modifier(component.as_str().to_string()))
                        .collect(),
                );
            }

            Rule::range => {
                let mut keys = vec![];
                for range_component in component.into_inner() {
                    match range_component.as_rule() {
                        Rule::keybind => {
                            keys.push(Token::Key(range_component.as_str().to_string()));
                        }
                        Rule::key_dashed_range => {
                            let mut bounds = range_component.into_inner();
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
                                eprintln!("lower and upper bounds are not ascii");
                                return;
                            }
                            assert!(lower_bound < upper_bound);

                            keys.extend(
                                (lower_bound..=upper_bound).map(|key| Token::Key(key.to_string())),
                            );
                        }
                        _ => {}
                    }
                }
                tokens.push(keys);
            }
            Rule::keybind => {
                tokens.push(vec![Token::Key(component.as_str().to_string())]);
            }
            Rule::command => {
                for subcomponent in component.into_inner() {
                    match subcomponent.as_rule() {
                        Rule::command_component => {
                            comm.push(vec![Token::Command(subcomponent.as_str().to_string())]);
                        }
                        Rule::command_with_brace => {
                            let mut command_variants = vec![];

                            for thing in subcomponent.into_inner() {
                                match thing.as_rule() {
                                    Rule::command_component => command_variants
                                        .push(Token::Command(thing.as_str().to_string())),
                                    Rule::dashed_range => {
                                        let mut bounds = thing.into_inner();
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
                                            eprintln!("lower and upper bounds are not ascii");
                                            return;
                                        }
                                        assert!(lower_bound < upper_bound);

                                        command_variants.extend(
                                            (lower_bound..=upper_bound)
                                                .map(|key| Token::Command(key.to_string())),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            comm.push(command_variants);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
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
    for content in parse_result {
        for decl in content
            .into_inner()
            .filter(|decl| decl.as_rule() == Rule::binding)
        {
            binding_parser(decl);
            println!("-----");
        }
    }
    Ok(())
}
