use std::collections::BTreeMap;

use super::{Call, CallKind, ClassFile, Method};

const CONSTRUCTOR: &str = "<init>";
const STATIC_INITIALIZER: &str = "<clinit>";
const RUNTIME: &str = "java/lang/Runtime";
const RECEIVER: &str = "receiver";

struct Names {
    own: String,
    own_class: String,
    imported: BTreeMap<String, String>,
}

pub fn skeleton(class: &ClassFile) -> String {
    let (package, own) = split(&class.name);
    let mut imported: BTreeMap<String, String> = BTreeMap::new();
    let referenced = class
        .methods
        .iter()
        .flat_map(|method| &method.calls)
        .map(|call| &call.class)
        .chain(&class.supertypes);
    for referenced_class in referenced {
        if referenced_class.starts_with('[') || *referenced_class == class.name {
            continue;
        }
        let (_, simple) = split(referenced_class);
        if simple != own {
            imported
                .entry(simple)
                .or_insert_with(|| referenced_class.clone());
        }
    }
    let names = Names {
        own: own.clone(),
        own_class: class.name.clone(),
        imported,
    };
    let mut lines = Vec::new();
    if !package.is_empty() {
        lines.push(format!("package {};", dotted(&package)));
    }
    lines.extend(
        names
            .imported
            .values()
            .map(|class| format!("import {};", dotted(class))),
    );
    let supertypes: Vec<String> = class
        .supertypes
        .iter()
        .map(|supertype| {
            reference_to(Reference {
                class: supertype,
                names: &names,
            })
        })
        .collect();
    lines.push(if supertypes.is_empty() {
        format!("class {} {{", identifier(&own))
    } else {
        format!(
            "class {} implements {} {{",
            identifier(&own),
            supertypes.join(", ")
        )
    });
    for method in &class.methods {
        lines.extend(method_lines(Rendered {
            method,
            names: &names,
        }));
    }
    lines.push("}".to_string());
    lines.join("\n") + "\n"
}

struct Rendered<'a> {
    method: &'a Method,
    names: &'a Names,
}

fn method_lines(rendered: Rendered) -> Vec<String> {
    let Rendered { method, names } = rendered;
    let header = match method.name.as_str() {
        STATIC_INITIALIZER => "static".to_string(),
        CONSTRUCTOR => format!("{}()", identifier(&names.own)),
        name if method.is_native => return vec![format!("native void {}();", identifier(name))],
        name => format!("void {}()", identifier(name)),
    };
    let mut lines = vec![format!("{header} {{")];
    for call in &method.calls {
        lines.extend(call_lines(CallIn { call, names }));
    }
    lines.push("}".to_string());
    lines
}

struct CallIn<'a> {
    call: &'a Call,
    names: &'a Names,
}

fn call_lines(input: CallIn) -> Vec<String> {
    let CallIn { call, names } = input;
    if call.class.starts_with('[') {
        return Vec::new();
    }
    let reference = reference_to(Reference {
        class: &call.class,
        names,
    });
    let name = identifier(&call.name);
    if call.name == CONSTRUCTOR {
        return vec![format!("new {reference}();")];
    }
    if call.class == RUNTIME {
        return vec![format!("Runtime.getRuntime().{name}();")];
    }
    let mut lines = vec![format!("{reference}.{name}();")];
    if call.kind == CallKind::Interface {
        lines.push(format!("{RECEIVER}.{name}();"));
    }
    lines
}

struct Reference<'a> {
    class: &'a str,
    names: &'a Names,
}

fn reference_to(wanted: Reference) -> String {
    let Reference { class, names } = wanted;
    let (_, simple) = split(class);
    if class == names.own_class {
        identifier(&names.own)
    } else if names.imported.get(&simple).map(String::as_str) == Some(class) {
        identifier(&simple)
    } else {
        dotted(class)
    }
}

fn split(class: &str) -> (String, String) {
    match class.rsplit_once('/') {
        Some((package, simple)) => (package.to_string(), simple.to_string()),
        None => (String::new(), class.to_string()),
    }
}

fn dotted(class: &str) -> String {
    class
        .split('/')
        .map(identifier)
        .collect::<Vec<_>>()
        .join(".")
}

fn identifier(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '$' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}
