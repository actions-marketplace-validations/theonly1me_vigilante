use super::declarations::{Declarations, type_name};
use crate::code::SyntaxNode;
use crate::deep::program::UseTarget;

pub const CONSTRUCTOR: &str = "constructor";

pub struct CallSite<'a, 'r> {
    pub node: &'a SyntaxNode<'r>,
    pub class: Option<&'a str>,
    pub declarations: &'a Declarations,
}

pub fn target_of(site: CallSite) -> Option<UseTarget> {
    let CallSite {
        node,
        class,
        declarations,
    } = site;
    match node.kind().as_ref() {
        "invocation_expression" => {
            let function = node.field("function")?;
            invoked(Invoked {
                function: &function,
                class,
                declarations,
            })
        }
        "object_creation_expression" => Some(member(Member {
            object: type_name(&node.field("type")?.text()),
            property: CONSTRUCTOR.to_string(),
        })),
        _ => None,
    }
}

struct Invoked<'a, 'r> {
    function: &'a SyntaxNode<'r>,
    class: Option<&'a str>,
    declarations: &'a Declarations,
}

fn invoked(input: Invoked) -> Option<UseTarget> {
    let Invoked {
        function,
        class,
        declarations,
    } = input;
    match function.kind().as_ref() {
        "identifier" | "generic_name" => {
            let name = simple_name(function)?;
            Some(match class {
                Some(class) => member(Member {
                    object: class.to_string(),
                    property: name,
                }),
                None => UseTarget::Name(name),
            })
        }
        "member_access_expression" => {
            let name = simple_name(&function.field("name")?)?;
            let expression = function.field("expression")?;
            let object = match expression.kind().as_ref() {
                "this" | "base" => class.map(str::to_string),
                "identifier" => {
                    let text = expression.text().to_string();
                    declarations
                        .variable_types
                        .get(&text)
                        .cloned()
                        .or_else(|| text.starts_with(char::is_uppercase).then_some(text))
                }
                "member_access_expression" | "qualified_name" => {
                    let text = expression.text().to_string();
                    let is_dotted = text.split('.').all(|part| {
                        !part.is_empty()
                            && part
                                .chars()
                                .all(|character| character.is_alphanumeric() || character == '_')
                    });
                    let first = text.split('.').next().unwrap_or_default();
                    (is_dotted && !declarations.variable_types.contains_key(first)).then_some(text)
                }
                _ => None,
            };
            Some(match object {
                Some(object) => member(Member {
                    object,
                    property: name,
                }),
                None => UseTarget::Method(name),
            })
        }
        _ => None,
    }
}

fn simple_name(node: &SyntaxNode) -> Option<String> {
    match node.kind().as_ref() {
        "identifier" => Some(node.text().to_string()),
        "generic_name" => node
            .children()
            .find(|child| child.kind() == "identifier")
            .map(|identifier| identifier.text().to_string()),
        _ => None,
    }
}

pub struct Member {
    pub object: String,
    pub property: String,
}

pub fn member(target: Member) -> UseTarget {
    UseTarget::Member {
        object: target.object,
        property: target.property,
        call: true,
    }
}
