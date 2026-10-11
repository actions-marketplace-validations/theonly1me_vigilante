use std::collections::BTreeSet;
use std::fmt::Write;

use super::{CallKind, MethodDefinition, TypeDefinition};

const CONSTRUCTOR: &str = ".ctor";
const STATIC_CONSTRUCTOR: &str = ".cctor";
const RECEIVER: &str = "receiver";

pub fn skeleton(definition: &TypeDefinition) -> String {
    let class = identifier(&definition.name);
    let namespaces: BTreeSet<&str> = definition
        .methods
        .iter()
        .flat_map(|method| &method.calls)
        .map(|call| call.namespace.as_str())
        .chain(
            definition
                .supertypes
                .iter()
                .map(|(namespace, _)| namespace.as_str()),
        )
        .filter(|namespace| !namespace.is_empty() && *namespace != definition.namespace)
        .collect();
    let mut text = String::new();
    for namespace in namespaces {
        let _ = writeln!(text, "using {namespace};");
    }
    if !definition.namespace.is_empty() {
        let _ = writeln!(text, "namespace {}\n{{", definition.namespace);
    }
    let bases: Vec<String> = definition
        .supertypes
        .iter()
        .map(|(_, name)| identifier(name))
        .collect();
    if bases.is_empty() {
        let _ = writeln!(text, "class {class}\n{{");
    } else {
        let _ = writeln!(text, "class {class} : {}\n{{", bases.join(", "));
    }
    for method in &definition.methods {
        text.push_str(&method_text(MethodText {
            method,
            class: &class,
        }));
    }
    text.push_str("}\n");
    if !definition.namespace.is_empty() {
        text.push_str("}\n");
    }
    text
}

struct MethodText<'a> {
    method: &'a MethodDefinition,
    class: &'a str,
}

fn method_text(input: MethodText) -> String {
    let MethodText { method, class } = input;
    if let Some(library) = &method.native_library {
        return format!(
            "[DllImport(\"{}\")] static extern void {}();\n",
            library.replace(['"', '\\'], "_"),
            identifier(&method.name)
        );
    }
    let header = match method.name.as_str() {
        CONSTRUCTOR => format!("{class}()"),
        STATIC_CONSTRUCTOR => format!("static {class}()"),
        name => format!("void {}()", identifier(name)),
    };
    let mut body = String::new();
    for call in &method.calls {
        let owner = identifier(&call.type_name);
        if call.method == CONSTRUCTOR || call.kind == CallKind::Create {
            let _ = writeln!(body, "new {owner}();");
            continue;
        }
        if call.method == STATIC_CONSTRUCTOR {
            continue;
        }
        let name = identifier(&call.method);
        let _ = writeln!(body, "{owner}.{name}();");
        if call.kind == CallKind::Virtual && looks_like_interface(&call.type_name) {
            let _ = writeln!(body, "{RECEIVER}.{name}();");
        }
    }
    format!("{header}\n{{\n{body}}}\n")
}

fn looks_like_interface(name: &str) -> bool {
    let mut characters = name.chars();
    characters.next() == Some('I') && characters.next().is_some_and(char::is_uppercase)
}

pub fn identifier(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.starts_with(|character: char| character.is_ascii_digit()) || cleaned.is_empty() {
        format!("_{cleaned}")
    } else {
        cleaned
    }
}
