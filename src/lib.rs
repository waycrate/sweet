use itertools::Itertools;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unable to parse grammar from invalid contents")]
    Grammar(#[from] pest::error::Error<Rule>),
    #[error("hotkey config must contain one and only one main section")]
    MainSection,
    #[error("shorthand lower bound `{0}` is not an ASCII character")]
    NonAsciiLowerBound(char),
    #[error("shorthand upper bound `{0}` is not an ASCII character")]
    NonAsciiUpperBound(char),
    #[error("shorthand lower bound `{0}` is greater than upper bound `{1}`")]
    InvalidBounds(char, char),
    #[error("the number of possible binding variants {0} does not equal the number of possible command variants {1}.")]
    UnequalShorthandLengths(usize, usize),
}

#[derive(Parser)]
#[grammar = "template.pest"]
pub struct SwhkdGrammar;

pub struct SwhkdParser {
    pub bindings: Vec<Binding>,
    pub unbinds: Vec<Definition>,
    pub imports: Vec<String>,
}

impl SwhkdParser {
    pub fn from(raw: &str) -> Result<Self, ParseError> {
        let parse_result = SwhkdGrammar::parse(Rule::main, raw)?;
        let Some(contents) = parse_result.into_iter().next() else {
            return Err(ParseError::MainSection);
        };

        let mut bindings = vec![];
        let mut unbinds = vec![];
        let mut imports = vec![];
        for decl in contents.into_inner() {
            match decl.as_rule() {
                Rule::binding => bindings.extend(binding_parser(decl)?),
                Rule::unbind => unbinds.extend(unbind_parser(decl)?),
                Rule::import => imports.extend(import_parser(decl)),
                // End of identifier
                // Here, it means the end of the file.
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }
        Ok(SwhkdParser {
            bindings,
            unbinds,
            imports,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Modifier(String),
    Key(String),
    Command(String),
}

#[derive(Debug)]
pub struct Definition(pub Vec<Token>);
#[derive(Debug)]
pub struct Command(pub Vec<Token>);
impl ToString for Command {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .filter_map(|t| {
                if let Token::Command(c) = t {
                    Some(c)
                } else {
                    None
                }
            })
            .join(" ")
    }
}

#[derive(Debug)]
pub struct Binding {
    pub definition: Definition,
    pub command: String,
}

trait ComponentToString {
    fn inner_string(&self) -> String;
}

impl ComponentToString for Pair<'_, Rule> {
    fn inner_string(&self) -> String {
        self.as_str().to_string()
    }
}

fn extract_trigger(component: Pair<'_, Rule>) -> Result<Vec<Token>, ParseError> {
    let trigger = match component.as_rule() {
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
                        let (lower_bound, upper_bound) = extract_bounds(shorthand_component)?;
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
    };
    Ok(trigger)
}

fn unbind_parser(pair: Pair<'_, Rule>) -> Result<Vec<Definition>, ParseError> {
    let mut unbind = vec![];
    for component in pair.into_inner() {
        unbind.push(extract_trigger(component)?);
    }
    let unbinds = unbind
        .into_iter()
        .multi_cartesian_product()
        .map(|d| Definition(d))
        .collect();

    Ok(unbinds)
}

fn import_parser(pair: Pair<'_, Rule>) -> Vec<String> {
    let mut imports = vec![];
    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::import_file => imports.push(component.inner_string()),
            _ => unreachable!(),
        }
    }
    imports
}

fn extract_bounds(pair: Pair<'_, Rule>) -> Result<(char, char), ParseError> {
    let mut bounds = pair.into_inner();

    // These unwraps must always work since the pest grammar picked up
    // the pairs because the lower and upper bounds were present to
    // begin with. These should not be categorized as errors.
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

    if !lower_bound.is_ascii() {
        return Err(ParseError::NonAsciiLowerBound(upper_bound));
    }
    if !upper_bound.is_ascii() {
        return Err(ParseError::NonAsciiUpperBound(upper_bound));
    }
    if lower_bound > upper_bound {
        return Err(ParseError::InvalidBounds(lower_bound, upper_bound));
    }
    Ok((lower_bound, upper_bound))
}

fn parse_command_shorthand(pair: Pair<'_, Rule>) -> Result<Vec<Token>, ParseError> {
    let mut command_variants = vec![];

    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::command_component => {
                command_variants.push(Token::Command(component.inner_string()))
            }
            Rule::range => {
                let (lower_bound, upper_bound) = extract_bounds(component)?;
                command_variants
                    .extend((lower_bound..=upper_bound).map(|key| Token::Command(key.to_string())));
            }
            _ => {}
        }
    }
    Ok(command_variants)
}

fn binding_parser(pair: Pair<'_, Rule>) -> Result<Vec<Binding>, ParseError> {
    let mut tokens = vec![];
    let mut comm = vec![];
    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::command => {
                for subcomponent in component.into_inner() {
                    match subcomponent.as_rule() {
                        Rule::command_standalone => {
                            comm.push(vec![Token::Command(subcomponent.inner_string())]);
                        }
                        Rule::command_shorthand => {
                            comm.push(parse_command_shorthand(subcomponent)?);
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                let trigger = extract_trigger(component)?;
                if !trigger.is_empty() {
                    tokens.push(trigger);
                }
            }
        }
    }
    let bind_cartesian_product: Vec<_> = tokens
        .into_iter()
        .multi_cartesian_product()
        .map(|b| Definition(b))
        .collect();
    let command_cartesian_product: Vec<_> = comm
        .into_iter()
        .multi_cartesian_product()
        .map(|c| Command(c).to_string())
        .collect();
    let bind_len = bind_cartesian_product.len();
    let command_len = command_cartesian_product.len();

    if bind_len != command_len {
        return Err(ParseError::UnequalShorthandLengths(bind_len, command_len));
    }

    let bindings = bind_cartesian_product
        .into_iter()
        .zip(command_cartesian_product.into_iter())
        .map(|(definition, command)| Binding {
            definition,
            command,
        })
        .collect();
    Ok(bindings)
}
