use super::writer::{TestCall, TestClass, TestMethod, class_bytes};
use super::{Call, CallKind, read, skeleton};

const SHELL: TestClass = TestClass {
    name: "com/acme/tools/Shell",
    methods: &[
        TestMethod {
            name: "<init>",
            is_native: false,
            calls: &[TestCall {
                class: "java/lang/Object",
                name: "<init>",
                kind: CallKind::Special,
            }],
            lambda: None,
        },
        TestMethod {
            name: "run",
            is_native: false,
            calls: &[
                TestCall {
                    class: "java/lang/ProcessBuilder",
                    name: "<init>",
                    kind: CallKind::Special,
                },
                TestCall {
                    class: "java/lang/ProcessBuilder",
                    name: "start",
                    kind: CallKind::Virtual,
                },
                TestCall {
                    class: "java/lang/Runtime",
                    name: "exec",
                    kind: CallKind::Virtual,
                },
                TestCall {
                    class: "com/acme/tools/Shell",
                    name: "helper",
                    kind: CallKind::Static,
                },
                TestCall {
                    class: "java/util/List",
                    name: "size",
                    kind: CallKind::Interface,
                },
            ],
            lambda: Some(TestCall {
                class: "com/acme/tools/Shell",
                name: "lambda$run$0",
                kind: CallKind::Static,
            }),
        },
        TestMethod {
            name: "attach",
            is_native: true,
            calls: &[],
            lambda: None,
        },
    ],
};

#[test]
fn a_class_file_lists_each_call_after_switches_and_operands() {
    let class = read(&class_bytes(&SHELL)).expect("class file");

    assert_eq!(class.name, "com/acme/tools/Shell");
    let run = class
        .methods
        .iter()
        .find(|method| method.name == "run")
        .expect("run");
    let expected: Vec<Call> = [
        ("java/lang/ProcessBuilder", "<init>", CallKind::Special),
        ("java/lang/ProcessBuilder", "start", CallKind::Virtual),
        ("java/lang/Runtime", "exec", CallKind::Virtual),
        ("com/acme/tools/Shell", "helper", CallKind::Static),
        ("java/util/List", "size", CallKind::Interface),
        ("com/acme/tools/Shell", "lambda$run$0", CallKind::Static),
    ]
    .into_iter()
    .map(|(class, name, kind)| Call {
        class: class.to_string(),
        name: name.to_string(),
        kind,
    })
    .collect();
    assert_eq!(run.calls, expected);
    assert!(
        class
            .methods
            .iter()
            .any(|method| method.name == "attach" && method.is_native)
    );
}

#[test]
fn a_skeleton_writes_java_that_the_pack_rules_can_read() {
    let class = read(&class_bytes(&SHELL)).expect("class file");

    let text = skeleton(&class);

    assert_eq!(
        text,
        "package com.acme.tools;\nimport java.util.List;\nimport java.lang.Object;\nimport java.lang.ProcessBuilder;\nimport java.lang.Runtime;\nclass Shell {\nShell() {\nnew Object();\n}\nvoid run() {\nnew ProcessBuilder();\nProcessBuilder.start();\nRuntime.getRuntime().exec();\nShell.helper();\nList.size();\nreceiver.size();\nShell.lambda$run$0();\n}\nnative void attach();\n}\n"
    );
}

#[test]
fn bytes_that_are_not_a_class_file_are_ignored() {
    assert_eq!(read(b"PK\x03\x04 not a class"), None);
    assert_eq!(read(&class_bytes(&SHELL)[..40]), None);
}
