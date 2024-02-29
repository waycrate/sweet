use anyhow::{bail, Result};
use std::{fmt::Debug, fs};

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template.pest"]
pub struct SwhkdParser;

fn dynamic_power_set_vec<T>(v: &mut Vec<Vec<T>>, append: &[T]) -> Vec<Vec<T>>
where
    T: AsRef<str> + Clone + Debug,
{
    let mut all_clones = vec![];
    for item in append {
        if v.is_empty() {
            all_clones.push(vec![item.clone()]);
            continue;
        }
        let mut v_clone = v.clone();
        for set in v_clone.iter_mut() {
            if !item.as_ref().eq("_") {
                set.push(item.clone());
            }
        }
        all_clones.extend(v_clone);
    }
    all_clones
}

fn binding_parser(pair: Pair<'_, Rule>) {
    let mut modifiers = vec![];
    let mut keysyms = vec![];
    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::modifier => {
                modifiers = dynamic_power_set_vec(&mut modifiers, &[component.as_str()]);
            }

            Rule::modifier_range => {
                let modifier: Vec<_> = component
                    .into_inner()
                    .map(|component| component.as_str())
                    .collect();
                modifiers = dynamic_power_set_vec(&mut modifiers, &modifier);
            }

            Rule::modifier_omit_range => {
                let modifier: Vec<_> = component
                    .into_inner()
                    .map(|component| component.as_str())
                    .collect();
                modifiers = dynamic_power_set_vec(&mut modifiers, &modifier);
            }

            Rule::range => {
                for range_component in component.into_inner() {
                    match range_component.as_rule() {
                        Rule::keybind => {
                            keysyms.push(range_component.as_str().to_string());
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

                            for key in lower_bound..=upper_bound {
                                keysyms.push(key.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            Rule::keybind => {
                keysyms.push(component.as_str().to_string());
            }
            _ => {}
        }
    }
    println!(
        "modifier cartesian product: {:#?}, keysyms: {:#?}",
        modifiers, keysyms
    );
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
