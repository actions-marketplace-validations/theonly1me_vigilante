use std::collections::HashMap;

use super::declarations::{Declarations, simple_type};
use crate::code::SyntaxNode;
use crate::deep::program::{Binding, UseTarget};

const THIS: &str = "this";
const CONSTRUCTOR: &str = "constructor";

pub struct UseIn<'a, 'r> {
    pub node: &'a SyntaxNode<'r>,
    pub declarations: &'a Declarations,
    pub bindings: &'a HashMap<String, Binding>,
}

pub fn use_of(input: UseIn) -> Option<UseTarget> {
    let UseIn {
        node,
        declarations,
        bindings,
    } = input;
    match node.kind().as_ref() {
        "method_invocation" => {
            let name = node.field("name")?.text().to_string();
            let Some(object) = node.field("object") else {
                return Some(if bindings.contains_key(&name) {
                    UseTarget::Name(name)
                } else {
                    member_call(MemberCall {
                        object: THIS.to_string(),
                        property: name,
                    })
                });
            };
            Some(
                match receiver(Receiver {
                    object: &object,
                    declarations,
                }) {
                    Some(owner) => member_call(MemberCall {
                        object: owner,
                        property: name,
                    }),
                    None => UseTarget::Method(name),
                },
            )
        }
        "method_reference" => {
            let mut parts = node.children().filter(|child| child.is_named());
            let object = parts.next()?;
            let name = parts.next()?.text().to_string();
            Some(
                match receiver(Receiver {
                    object: &object,
                    declarations,
                }) {
                    Some(owner) => member_call(MemberCall {
                        object: owner,
                        property: name,
                    }),
                    None => UseTarget::Method(name),
                },
            )
        }
        "object_creation_expression" => Some(member_call(MemberCall {
            object: simple_type(&node.field("type")?.text()),
            property: CONSTRUCTOR.to_string(),
        })),
        _ => None,
    }
}

struct Receiver<'a, 'r> {
    object: &'a SyntaxNode<'r>,
    declarations: &'a Declarations,
}

fn receiver(input: Receiver) -> Option<String> {
    let Receiver {
        object,
        declarations,
    } = input;
    let text = object.text().to_string();
    match object.kind().as_ref() {
        "this" | "super" => Some(THIS.to_string()),
        "identifier" | "type_identifier" => declarations
            .variable_types
            .get(&text)
            .cloned()
            .or_else(|| text.starts_with(char::is_uppercase).then_some(text)),
        "field_access" | "scoped_identifier" => {
            let field = object
                .field("field")
                .or_else(|| object.field("name"))?
                .text()
                .to_string();
            let owner = object.field("object").or_else(|| object.field("scope"))?;
            if owner.kind() == "this" {
                return declarations.variable_types.get(&field).cloned();
            }
            let is_qualified = text.split('.').all(|part| {
                part.chars().all(|character| {
                    character.is_alphanumeric() || character == '_' || character == '$'
                })
            });
            is_qualified.then_some(text)
        }
        _ => None,
    }
}

struct MemberCall {
    object: String,
    property: String,
}

fn member_call(target: MemberCall) -> UseTarget {
    UseTarget::Member {
        object: target.object,
        property: target.property,
        call: true,
    }
}
