use anyhow::{bail, Result};
use pester::SwhkdParser;
use std::fs;

fn main() -> Result<()> {
    let Some(arg) = std::env::args().nth(1) else {
        bail!("please supply a path to a hotkeys config file");
    };
    let raw_content = fs::read_to_string(arg)?;
    let parser = SwhkdParser::from(&raw_content)?;

    println!("bindings: {:?}", parser.bindings);
    println!("unbinds: {:?}", parser.unbinds);
    println!("imports: {:?}", parser.imports);
    Ok(())
}
