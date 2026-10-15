use crate::code::SyntaxNode;

pub struct Definable<'a, 'r> {
    pub node: &'a SyntaxNode<'r>,
    pub class: Option<&'a str>,
}

struct Owned<'a, 'r> {
    object: &'a SyntaxNode<'r>,
    class: Option<&'a str>,
}

struct Qualified<'a> {
    owner: Option<&'a str>,
    name: &'a str,
}

pub fn definition_name(definable: Definable) -> Option<(String, String)> {
    let Definable { node, class } = definable;
    let own_name = node.field("name").map(|name| plain(&name.text()));
    if node.kind() == "method_definition" {
        let simple = own_name?;
        let owner = match class {
            Some(class) => Some(class.to_string()),
            None => node.parent().and_then(|object| holder_name(&object)),
        };
        return Some((
            qualified(Qualified {
                owner: owner.as_deref(),
                name: &simple,
            }),
            simple,
        ));
    }
    if node.kind() == "function_declaration" || node.kind() == "generator_function_declaration" {
        let simple = own_name?;
        return Some((simple.clone(), simple));
    }
    let parent = node.parent()?;
    let simple = match parent.kind().as_ref() {
        "variable_declarator" => name_of_identifier(&parent.field("name")?)?,
        "assignment_expression" => {
            let left = parent.field("left")?;
            match left.kind().as_ref() {
                "identifier" => left.text().to_string(),
                "member_expression" if left.text() == "module.exports" => {
                    own_name.unwrap_or_else(|| "default".to_string())
                }
                "member_expression" => {
                    let property = plain(&left.field("property")?.text());
                    let owner = left.field("object").and_then(|object| {
                        owner_of(Owned {
                            object: &object,
                            class,
                        })
                    });
                    return Some((
                        qualified(Qualified {
                            owner: owner.as_deref(),
                            name: &property,
                        }),
                        property,
                    ));
                }
                _ => return None,
            }
        }
        "pair" => {
            let key = plain(&parent.field("key")?.text());
            let owner = parent.parent().and_then(|object| holder_name(&object));
            return Some((
                qualified(Qualified {
                    owner: owner.as_deref(),
                    name: &key,
                }),
                key,
            ));
        }
        "public_field_definition" => {
            let field = plain(&parent.field("name")?.text());
            return Some((
                qualified(Qualified {
                    owner: class,
                    name: &field,
                }),
                field,
            ));
        }
        "export_statement" => "default".to_string(),
        _ => own_name?,
    };
    Some((simple.clone(), simple))
}

pub fn class_name(node: &SyntaxNode) -> String {
    if let Some(name) = node.field("name") {
        return name.text().to_string();
    }
    node.parent()
        .filter(|parent| parent.kind() == "variable_declarator")
        .and_then(|parent| parent.field("name"))
        .map_or_else(|| "default".to_string(), |name| name.text().to_string())
}

fn holder_name(object: &SyntaxNode) -> Option<String> {
    let holder = object.parent()?;
    match holder.kind().as_ref() {
        "variable_declarator" => name_of_identifier(&holder.field("name")?),
        "assignment_expression" => {
            let left = holder.field("left")?;
            match left.text().as_ref() {
                "module.exports" | "exports" => None,
                _ => owner_of(Owned {
                    object: &left,
                    class: None,
                }),
            }
        }
        _ => None,
    }
}

fn owner_of(owned: Owned) -> Option<String> {
    let Owned { object, class } = owned;
    let text = object.text();
    match text.as_ref() {
        "exports" | "module.exports" => None,
        "this" => class.map(str::to_string),
        _ => Some(
            text.trim_end_matches(".prototype")
                .rsplit('.')
                .next()
                .unwrap_or_default()
                .to_string(),
        ),
    }
}

fn name_of_identifier(node: &SyntaxNode) -> Option<String> {
    (node.kind() == "identifier").then(|| node.text().to_string())
}

fn qualified(qualified: Qualified) -> String {
    let Qualified { owner, name } = qualified;
    match owner {
        Some(owner) if !owner.is_empty() => format!("{owner}.{name}"),
        _ => name.to_string(),
    }
}

fn plain(text: &str) -> String {
    text.trim_matches(['"', '\'', '`', '[', ']'])
        .trim_start_matches('#')
        .to_string()
}
