use anyhow::{bail, Result};
use std::fs;
use sweet::SwhkdParser;

fn main() -> Result<()> {
    let Some(arg) = std::env::args().nth(1) else {
        bail!("please supply a path to a hotkeys config file");
    };
    let raw_content = fs::read_to_string(arg)?;
    let parser = SwhkdParser::from(&raw_content)?;

    for binding in parser.bindings {
        println!("binding: {:?}", binding);
    }
    for unbind in parser.unbinds {
        println!("unbind: {:?}", unbind);
    }
    for import in parser.imports {
        println!("import: {:?}", import);
    }
    Ok(())
}
