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

    pub fn texts(&self) -> Vec<Located<'_>> {
        let mut found = Vec::new();
        self.collect_texts(&mut found);
        found
    }

    fn collect_texts<'a>(&'a self, found: &mut Vec<Located<'a>>) {
        match &self.value {
            Value::Empty => {}
            Value::Text(text) => found.push(Located {
                text,
                line: self.line,
            }),
            Value::List(items) => items.iter().for_each(|item| item.collect_texts(found)),
            Value::Map(entries) => {
                for entry in entries {
                    found.push(Located {
                        text: &entry.key,
                        line: entry.line,
                    });
                    entry.value.collect_texts(found);
                }
            }
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
