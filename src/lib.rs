use itertools::Itertools;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unable to parse grammar from invalid contents")]
    // pest::error::Error being 184 bytes makes this entire enum
    // expensive to copy, hence the box is used to put it on the heap.
    Grammar(#[from] Box<pest::error::Error<Rule>>),
    #[error("hotkey config must contain one and only one main section")]
    MainSection,
    #[error(
        "binding definitions must have a trailing regular key prefixed by one or more modifiers"
    )]
    Definition,
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
        let parse_result = SwhkdGrammar::parse(Rule::main, raw)
            .map_err(|err| ParseError::Grammar(Box::new(err)))?;
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
    Modifier(Modifier),
    Key(Key),
}

#[derive(Debug, Clone)]
pub struct Modifier(String);
#[derive(Debug, Clone)]
pub struct Key {
    key: String,
    attribute: KeyAttribute,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct KeyAttribute: u8 {
        const None = 0b00000000;
        const Send = 0b00000001;
        const OnRelease = 0b00000010;
        const Both = Self::Send.bits() | Self::OnRelease.bits();
    }
}

#[derive(Debug)]
pub struct Definition {
    modifiers: Vec<Modifier>,
    key: Key,
}

impl TryFrom<Vec<Token>> for Definition {
    type Error = ParseError;
    fn try_from(value: Vec<Token>) -> Result<Self, Self::Error> {
        let Some((key, modifiers_wrapped)) = value.split_last() else {
            // This error is incredibly unlikely since the grammar
            // picks up the binding definition because the constraints are met.
            return Err(ParseError::Definition);
        };
        let mut modifiers = vec![];
        for m in modifiers_wrapped.into_iter() {
            if let Token::Modifier(modifier) = m {
                modifiers.push(modifier.clone());
            } else {
                return Err(ParseError::Definition);
            }
        }
        let Token::Key(key) = key.clone() else {
            return Err(ParseError::Definition);
        };
        Ok(Definition { modifiers, key })
    }
}

#[derive(Debug)]
pub struct Binding {
    pub definition: Definition,
    pub command: String,
}

fn pair_to_string(pair: Pair<'_, Rule>) -> String {
    pair.as_str().to_string()
}

fn parse_key(component: Pair<'_, Rule>) -> Token {
    let mut attribute = KeyAttribute::None;
    let mut key = String::default();
    for inner in component.into_inner() {
        match inner.as_rule() {
            Rule::send => attribute |= KeyAttribute::Send,
            Rule::on_release => attribute |= KeyAttribute::OnRelease,
            Rule::key => key = pair_to_string(inner),
            _ => {}
        }
    }
    Token::Key(Key { key, attribute })
}

fn extract_trigger(component: Pair<'_, Rule>) -> Result<Vec<Token>, ParseError> {
    let trigger = match component.as_rule() {
        Rule::modifier => vec![Token::Modifier(Modifier(pair_to_string(component)))],
        Rule::modifier_shorthand | Rule::modifier_omit_shorthand => component
            .into_inner()
            .map(|component| Token::Modifier(Modifier(pair_to_string(component))))
            .collect(),
        Rule::shorthand => {
            let mut keys = vec![];
            for shorthand_component in component.into_inner() {
                match shorthand_component.as_rule() {
                    Rule::key_in_shorthand => keys.push(parse_key(shorthand_component)),
                    Rule::key_range => {
                        let (lower_bound, upper_bound) = extract_bounds(shorthand_component)?;
                        keys.extend((lower_bound..=upper_bound).map(|key| {
                            Token::Key(Key {
                                key: key.to_string(),
                                attribute: KeyAttribute::None,
                            })
                        }));
                    }
                    _ => {}
                }
            }
            keys
        }
        Rule::key_in_shorthand | Rule::key_normal => vec![parse_key(component)],
        _ => vec![],
    };
    Ok(trigger)
}

fn unbind_parser(pair: Pair<'_, Rule>) -> Result<Vec<Definition>, ParseError> {
    let mut unbind = vec![];
    for component in pair.into_inner() {
        unbind.push(extract_trigger(component)?);
    }
    let unbinds = unbind.into_iter().multi_cartesian_product();

    let mut defs = vec![];
    for unbind in unbinds {
        defs.push(unbind.try_into()?);
    }

    Ok(defs)
}

fn import_parser(pair: Pair<'_, Rule>) -> Vec<String> {
    pair.into_inner()
        .filter(|component| matches!(component.as_rule(), Rule::import_file))
        .map(|component| pair_to_string(component))
        .collect()
}

fn extract_bounds(pair: Pair<'_, Rule>) -> Result<(char, char), ParseError> {
    let mut bounds = pair.clone().into_inner();

    // These unwraps must always work since the pest grammar picked up
    // the pairs due to the presence of the lower and upper bounds.
    // These should not be categorized as errors.
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
        let err = pest::error::Error::new_from_span(
            pest::error::ErrorVariant::<Rule>::CustomError {
                message: format!(
                    "shorthand lower bound `{0}` is not an ASCII character",
                    lower_bound
                ),
            },
            pair.as_span(),
        );
        return Err(Box::new(err).into());
    }
    if !upper_bound.is_ascii() {
        let err = pest::error::Error::new_from_span(
            pest::error::ErrorVariant::<Rule>::CustomError {
                message: format!(
                    "shorthand upper bound `{0}` is not an ASCII character",
                    upper_bound
                ),
            },
            pair.as_span(),
        );
        return Err(Box::new(err).into());
    }
    if lower_bound > upper_bound {
        let err = pest::error::Error::new_from_span(
            pest::error::ErrorVariant::<Rule>::CustomError {
                message: format!(
                    "shorthand lower bound `{}` is greater than upper bound `{}`",
                    lower_bound, upper_bound
                ),
            },
            pair.as_span(),
        );
        return Err(Box::new(err).into());
    }
    Ok((lower_bound, upper_bound))
}

fn parse_command_shorthand(pair: Pair<'_, Rule>) -> Result<Vec<String>, ParseError> {
    let mut command_variants = vec![];

    for component in pair.into_inner() {
        match component.as_rule() {
            Rule::command_component => command_variants.push(pair_to_string(component)),
            Rule::range => {
                let (lower_bound, upper_bound) = extract_bounds(component)?;
                command_variants.extend((lower_bound..=upper_bound).map(|key| key.to_string()));
            }
            _ => {}
        }
    }
    Ok(command_variants)
}

fn binding_parser(pair: Pair<'_, Rule>) -> Result<Vec<Binding>, ParseError> {
    let mut tokens = vec![];
    let mut comm = vec![];
    for component in pair.clone().into_inner() {
        match component.as_rule() {
            Rule::command => {
                for subcomponent in component.into_inner() {
                    match subcomponent.as_rule() {
                        Rule::command_standalone => {
                            comm.push(vec![pair_to_string(subcomponent)]);
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
        .map(Definition::try_from)
        .collect::<Result<Vec<Definition>, ParseError>>()?;
    let command_cartesian_product: Vec<_> = comm
        .into_iter()
        .multi_cartesian_product()
        .map(|c| c.join(" "))
        .collect();
    let bind_len = bind_cartesian_product.len();
    let command_len = command_cartesian_product.len();

    if bind_len != command_len {
        let err = pest::error::Error::new_from_span(
            pest::error::ErrorVariant::<Rule>::CustomError {
                message: format!(
                    "the number of possible binding variants {0} does not equal the number of possible command variants {1}.",
                    bind_len, command_len
                ),
            },
            pair.as_span(),
        );
        return Err(Box::new(err).into());
    }

    let bindings = bind_cartesian_product
        .into_iter()
        .zip(command_cartesian_product)
        .map(|(definition, command)| Binding {
            definition,
            command,
        })
        .collect();
    Ok(bindings)
}
