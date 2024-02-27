use anyhow::{bail, Result};
use std::fs;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "template.pest"]
pub struct SwhkdParser;

fn binding_parser(pair: Pair<'_, Rule>) {
    for component in pair.into_inner() {
        println!("{:#?}", component.as_rule());
        match component.as_rule() {
            Rule::modifier => {
                println!("{}", component.as_str());
            }

            Rule::modifier_range => {
                let modifiers: Vec<_> = component
                    .into_inner()
                    .map(|component| component.as_str())
                    .collect(); // The grammar only allows discrete values of modifiers.
                                // We will not support ranges for them.
                println!("{:#?}", modifiers);
            }
            _ => {}
        }
        println!("-----");
    }
}

fn main() -> Result<()> {
    let Some(arg) = std::env::args().skip(1).next() else {
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
