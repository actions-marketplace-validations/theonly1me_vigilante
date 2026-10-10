use saphyr::{LoadableYamlNode, MarkedYaml, Scalar, YamlData};

use crate::Error;

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub line: usize,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Empty,
    Text(String),
    List(Vec<Node>),
    Map(Vec<Entry>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: String,
    pub line: usize,
    pub value: Node,
}

pub struct Located<'a> {
    pub text: &'a str,
    pub line: usize,
}

impl Node {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let documents =
            MarkedYaml::load_from_str(text).map_err(|error| Error::new(error.to_string()))?;
        Ok(documents.first().map_or(
            Self {
                line: 1,
                value: Value::Empty,
            },
            convert,
        ))
    }

    pub fn parse_json(text: &str) -> Result<Self, Error> {
        Self::parse(&without_json_comments(text))
    }

    pub fn get(&self, key: &str) -> Option<&Node> {
        self.entries()
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.value)
    }

    pub fn entries(&self) -> &[Entry] {
        match &self.value {
            Value::Map(entries) => entries,
            _ => &[],
        }
    }

    pub fn items(&self) -> &[Node] {
        match &self.value {
            Value::List(items) => items,
            _ => &[],
        }
    }

    pub fn text(&self) -> Option<&str> {
        match &self.value {
            Value::Text(text) => Some(text),
            _ => None,
        }
    }
}

fn convert(node: &MarkedYaml) -> Node {
    let value = match &node.data {
        YamlData::Value(Scalar::Null) | YamlData::Alias(_) | YamlData::BadValue => Value::Empty,
        YamlData::Value(scalar) => Value::Text(scalar_text(scalar)),
        YamlData::Representation(text, _, _) => Value::Text(text.to_string()),
        YamlData::Sequence(items) => Value::List(items.iter().map(convert).collect()),
        YamlData::Mapping(mapping) => Value::Map(
            mapping
                .iter()
                .map(|(key, value)| Entry {
                    key: convert(key).text().unwrap_or_default().to_string(),
                    line: key.span.start.line(),
                    value: convert(value),
                })
                .collect(),
        ),
        YamlData::Tagged(_, inner) => convert(inner).value,
    };
    Node {
        line: node.span.start.line(),
        value,
    }
}

fn scalar_text(scalar: &Scalar) -> String {
    match scalar {
        Scalar::Null => String::new(),
        Scalar::Boolean(value) => value.to_string(),
        Scalar::Integer(value) => value.to_string(),
        Scalar::FloatingPoint(value) => value.to_string(),
        Scalar::String(value) => value.to_string(),
    }
}

fn without_json_comments(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    let mut in_string = false;
    while let Some(character) = characters.next() {
        if in_string {
            output.push(character);
            if character == '\\' {
                output.extend(characters.next());
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match (character, characters.peek()) {
            ('"', _) => {
                in_string = true;
                output.push(character);
            }
            ('/', Some('/')) => {
                while characters.next_if(|next| *next != '\n').is_some() {
                    output.push(' ');
                }
                output.push(' ');
            }
            ('/', Some('*')) => {
                characters.next();
                output.push_str("  ");
                while let Some(inner) = characters.next() {
                    if inner == '*' && characters.next_if_eq(&'/').is_some() {
                        output.push_str("  ");
                        break;
                    }
                    output.push(if inner == '\n' { '\n' } else { ' ' });
                }
            }
            _ => output.push(character),
        }
    }
    output
}
