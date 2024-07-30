use std::fmt::Display;

use crate::{Definition, ModeInstruction};

#[derive(Debug, PartialEq, Eq)]
pub struct Binding {
    pub definition: Definition,
    pub command: String,
    pub mode_instructions: Vec<ModeInstruction>,
}

impl Binding {
    pub fn running<S: AsRef<str>>(command: S) -> BindingBuilder {
        BindingBuilder {
            command: command.as_ref().to_string(),
        }
    }
}

pub struct BindingBuilder {
    pub command: String,
}

impl BindingBuilder {
    pub fn on(self, definition: Definition) -> Binding {
        Binding {
            definition,
            command: self.command,
            mode_instructions: vec![],
        }
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Binding {} \u{2192} {} (mode instructions: {:?})",
            self.definition, self.command, self.mode_instructions
        )
    }
}
