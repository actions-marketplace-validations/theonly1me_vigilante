use std::collections::HashMap;

use crate::code::SyntaxNode;
use crate::deep::program::UseTarget;

pub const CONSTRUCTOR: &str = "new";
pub const LOADERS: &[&str] = &["require", "require_relative", "load"];
const RELATIVE_LOADER: &str = "require_relative";

pub struct CallSite<'a, 'r> {
    pub node: &'a SyntaxNode<'r>,
    pub class: Option<&'a str>,
    pub variables: &'a HashMap<String, String>,
}

pub fn call_target(site: CallSite) -> Option<UseTarget> {
    let CallSite {
        node,
        class,
        variables,
    } = site;
    let method = node.field("method")?.text().to_string();
    let object = match node.field("receiver") {
        None => return Some(own_call(OwnCall { class, method })),
        Some(receiver) => match receiver.kind().as_ref() {
            "self" => class.map(str::to_string),
            "constant" | "scope_resolution" => Some(receiver.text().to_string()),
            "identifier" | "instance_variable" => variables.get(receiver.text().as_ref()).cloned(),
            _ => None,
        },
    };
    Some(match object {
        Some(object) => member(Member {
            object,
            property: method,
        }),
        None => UseTarget::Method(method),
    })
}

pub fn loaded(node: &SyntaxNode) -> Option<String> {
    if node.kind() != "call" || node.field("receiver").is_some() {
        return None;
    }
    let method = node.field("method")?.text().to_string();
    if !LOADERS.contains(&method.as_str()) {
        return None;
    }
    let path = node
        .field("arguments")?
        .children()
        .find(|argument| argument.kind() == "string")?
        .children()
        .find(|part| part.kind() == "string_content")?
        .text()
        .to_string();
    Some(if method == RELATIVE_LOADER && !path.starts_with('.') {
        format!("./{path}")
    } else {
        path
    })
}

pub struct OwnCall<'a> {
    pub class: Option<&'a str>,
    pub method: String,
}

pub fn own_call(call: OwnCall) -> UseTarget {
    match call.class {
        Some(class) => member(Member {
            object: class.to_string(),
            property: call.method,
        }),
        None => UseTarget::Name(call.method),
    }
}

struct Member {
    object: String,
    property: String,
}

fn member(target: Member) -> UseTarget {
    UseTarget::Member {
        object: target.object,
        property: target.property,
        call: true,
    }
}

pub fn variable_types(root: &SyntaxNode) -> HashMap<String, String> {
    root.dfs()
        .filter(|node| node.kind() == "assignment")
        .filter_map(|node| {
            let left = node.field("left")?;
            let right = node.field("right")?;
            if !matches!(left.kind().as_ref(), "identifier" | "instance_variable")
                || right.kind() != "call"
                || right.field("method")?.text() != CONSTRUCTOR
            {
                return None;
            }
            let receiver = right.field("receiver")?;
            matches!(receiver.kind().as_ref(), "constant" | "scope_resolution")
                .then(|| (left.text().to_string(), receiver.text().to_string()))
        })
        .collect()
}
