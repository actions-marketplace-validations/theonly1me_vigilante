use super::{Assembly, CallKind, MethodDefinition, TypeDefinition, read, skeleton};

const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/csharp/Acme.Tools.dll"
));

fn assembly() -> Assembly {
    read(FIXTURE).expect("assembly")
}

fn type_named<'a>(assembly: &'a Assembly) -> impl Fn(&str) -> &'a TypeDefinition {
    move |name| {
        assembly
            .types
            .iter()
            .find(|definition| definition.name == name)
            .unwrap_or_else(|| panic!("type {name}"))
    }
}

fn calls_of(method: &MethodDefinition) -> Vec<String> {
    method
        .calls
        .iter()
        .map(|call| {
            format!(
                "{:?} {}.{}.{}",
                call.kind, call.namespace, call.type_name, call.method
            )
        })
        .collect()
}

fn method<'a>(definition: &'a TypeDefinition) -> impl Fn(&str) -> &'a MethodDefinition {
    move |name| {
        definition
            .methods
            .iter()
            .find(|method| method.name == name)
            .unwrap_or_else(|| panic!("method {name}"))
    }
}

#[test]
fn calls_after_a_switch_and_wide_operands_are_read() {
    let assembly = assembly();
    let shell = type_named(&assembly)("Shell");

    let calls = calls_of(method(shell)("Run"));

    assert_eq!(shell.namespace, "Acme.Tools");
    assert!(
        calls.contains(&"Direct System.Diagnostics.Process.Start".to_string()),
        "{calls:?}"
    );
    assert!(
        calls.contains(&"Direct System.Console.WriteLine".to_string()),
        "{calls:?}"
    );
}

#[test]
fn interface_generic_and_lambda_calls_are_read() {
    let assembly = assembly();
    let caller = type_named(&assembly)("Caller");

    let calls = calls_of(method(caller)("Call"));

    assert!(
        calls.contains(&"Virtual Acme.Tools.IRunner.Go".to_string()),
        "{calls:?}"
    );
    assert!(
        calls.contains(&"Create System.Collections.Generic.List..ctor".to_string()),
        "{calls:?}"
    );
    assert!(
        calls.contains(&"Virtual System.Collections.Generic.List.Add".to_string()),
        "{calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|call| call.starts_with("Pointer Acme.Tools.<>c.<Call>b__")),
        "{calls:?}"
    );
}

#[test]
fn native_methods_nested_types_and_static_constructors_are_read() {
    let assembly = assembly();
    let native = type_named(&assembly)("Native");
    let inner = type_named(&assembly)("Inner");
    let settings = type_named(&assembly)("Settings");

    assert_eq!(
        method(native)("getpid").native_library.as_deref(),
        Some("libc")
    );
    assert_eq!(method(native)("Pid").native_library, None);
    assert_eq!(inner.enclosing.as_deref(), Some("Outer"));
    assert_eq!(inner.namespace, "Acme.Tools");
    assert!(
        calls_of(method(settings)(".cctor")).contains(&"Direct Acme.Tools.Shell.Run".to_string())
    );
    assert!(calls_of(method(inner)("Work")).contains(&"Direct Acme.Tools.Shell.Run".to_string()));
    let _ = CallKind::Direct;
}

#[test]
fn a_class_lists_its_interfaces_and_leaves_out_object() {
    let assembly = assembly();
    let named = type_named(&assembly);

    assert_eq!(
        named("Runner").supertypes,
        vec![("Acme.Tools".to_string(), "IRunner".to_string())]
    );
    assert_eq!(named("Caller").supertypes, Vec::<(String, String)>::new());
}

#[test]
fn bytes_that_are_not_an_assembly_are_ignored() {
    assert!(read(b"MZ not really a portable executable").is_none());
    assert!(read(&FIXTURE[..200]).is_none());
}

#[test]
fn a_skeleton_writes_csharp_that_the_pack_rules_can_read() {
    let assembly = assembly();
    let skeleton_of = |name: &str| skeleton(type_named(&assembly)(name));

    let shell = skeleton_of("Shell");
    let caller = skeleton_of("Caller");
    let native = skeleton_of("Native");
    let settings = skeleton_of("Settings");

    assert!(shell.contains("using System.Diagnostics;"), "{shell}");
    assert!(shell.contains("namespace Acme.Tools"), "{shell}");
    assert!(shell.contains("void Run()"), "{shell}");
    assert!(shell.contains("Process.Start();"), "{shell}");
    assert!(caller.contains("IRunner.Go();"), "{caller}");
    assert!(caller.contains("receiver.Go();"), "{caller}");
    assert!(caller.contains("new List();"), "{caller}");
    assert!(!caller.contains("receiver.Add();"), "{caller}");
    assert!(caller.contains("__c._Call_b__"), "{caller}");
    assert!(
        native.contains("[DllImport(\"libc\")] static extern void getpid();"),
        "{native}"
    );
    assert!(settings.contains("static Settings()"), "{settings}");
}
