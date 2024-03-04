use anyhow::{bail, Result};
use itertools::Itertools;
use std::{fmt::Debug, fs};

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template.pest"]
pub struct SwhkdParser;

fn dynamic_power_set_vec<T>(v: &[Vec<T>], append: &[T]) -> Vec<Vec<T>>
where
    T: AsRef<str> + Clone,
{
    let mut all_clones = Vec::with_capacity(v.len() * append.len());

    for item in append {
        if v.is_empty() {
            all_clones.push(vec![item.clone()]);
            continue;
        }

        for set in v {
            let mut new_set = set.clone();
            if !item.as_ref().eq("_") {
                new_set.push(item.clone());
            }
            all_clones.push(new_set);
        }
    }

    all_clones
}

#[derive(Debug)]
pub enum Token {
    Modifier(String),
    Key(String),
    Command(String),
}

fn binding_parser(pair: Pair<'_, Rule>) {
    let mut tokens = vec![];
    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::modifier => {
                tokens.push(vec![Token::Modifier(component.as_str().to_string())]);
            }

            Rule::modifier_range => {
                tokens.push(
                    component
                        .into_inner()
                        .map(|component| Token::Modifier(component.as_str().to_string()))
                        .collect(),
                );
            }

            Rule::modifier_omit_range => {
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
            _ => {}
        }
    }
    let multi_cartesian_product: Vec<_> = tokens.iter().multi_cartesian_product().collect();
    println!("multi_cartesian_product: {:#?}", multi_cartesian_product);
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
