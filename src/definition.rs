use itertools::Itertools;
use pest::iterators::Pair;

use crate::{
    pair_to_string, parse_key,
    range::Bounds,
    token::{Key, KeyAttribute, Modifier},
    KeyRepr, ModifierRepr, ParseError, Rule,
};
use std::{collections::BTreeSet, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub struct Definition {
    pub modifiers: BTreeSet<Modifier>,
    pub key: Key,
}

impl Definition {
    pub fn new(key: evdev::Key) -> Self {
        Self {
            modifiers: BTreeSet::default(),
            key: Key::new(key, KeyAttribute::None),
        }
    }

    pub fn with_modifiers(mut self, modifiers: &[Modifier]) -> Self {
        self.modifiers = modifiers.iter().cloned().collect();
        self
    }
}

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for modifier in self.modifiers.iter() {
            write!(f, "{:?}, ", modifier)?;
        }
        write!(f, "{:?}", self.key)?;
        write!(f, "]")
    }
}

#[derive(Default)]
pub struct DefinitionUncompiled {
    pub modifiers: Vec<Vec<Modifier>>,
    pub keys: Vec<Key>,
}

impl DefinitionUncompiled {
    pub fn ingest(&mut self, component: Pair<'_, Rule>) -> Result<(), ParseError> {
        match component.as_rule() {
            Rule::modifier => {
                self.modifiers.push(vec![
                    ModifierRepr(pair_to_string(component).to_lowercase()).into()
                ])
            }
            Rule::modifier_shorthand | Rule::modifier_omit_shorthand => self.modifiers.push(
                component
                    .into_inner()
                    .map(|component| ModifierRepr(pair_to_string(component)).into())
                    .collect(),
            ),
            Rule::shorthand => {
                for shorthand_component in component.into_inner() {
                    match shorthand_component.as_rule() {
                        Rule::key_in_shorthand => {
                            self.keys.push(parse_key(shorthand_component).try_into()?)
                        }
                        Rule::key_range => {
                            let (lower_bound, upper_bound) =
                                Bounds::new(shorthand_component).expand_keys()?;
                            let keys = (lower_bound..=upper_bound)
                                .map(|key| {
                                    KeyRepr {
                                        key: key.to_string(),
                                        attribute: KeyAttribute::None,
                                    }
                                    .try_into()
                                })
                                .collect::<Result<Vec<Key>, ParseError>>()?;
                            self.keys.extend(keys);
                        }
                        _ => {}
                    }
                }
            }
            Rule::key_normal => self.keys.push(parse_key(component).try_into()?),
            _ => {}
        };
        Ok(())
    }

    pub fn compile(self) -> Vec<Definition> {
        if self.modifiers.is_empty() {
            return self
                .keys
                .into_iter()
                .map(|key| Definition {
                    modifiers: BTreeSet::default(),
                    key,
                })
                .collect();
        }
        self.modifiers
            .into_iter()
            .multi_cartesian_product()
            .cartesian_product(self.keys)
            .map(|(modifiers, key)| Definition {
                modifiers: modifiers.into_iter().collect(),
                key,
            })
            .collect()
    }
}
