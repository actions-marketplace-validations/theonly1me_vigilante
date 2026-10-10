use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::SupportLang;

use super::index;
use crate::csharp::metadata::{read, skeleton};
use crate::deep::program::{Binding, Definition, Export, FileIndex, Imported, UseTarget};

const SOURCE: &str = r#"using System.Net.Http;
using Json = Newtonsoft.Json.JsonConvert;

namespace Acme.App;

public partial class Worker : Base
{
    private readonly HttpClient client = new HttpClient();
    private readonly Tools.Shell shell;

    static Worker() { Setup(); }

    public Worker(IRunner runner) { runner.Go(); }

    public string Name { get { return Load(); } }

    public void Run(int count)
    {
        var report = new Report();
        this.Helper();
        Helper();
        Acme.Tools.Shell.Run(1);
        shell.Execute().Wait();
        report.Save();
        Json.SerializeObject(report);
        Action later = () => Fetch();
    }
}
"#;

fn indexed(source: &str) -> FileIndex {
    let document = SupportLang::CSharp.ast_grep(source);
    index(&document.root())
}

fn targets_of(definition: Option<&Definition>) -> Vec<String> {
    definition
        .map(|definition| {
            definition
                .uses
                .iter()
                .map(|item| match &item.target {
                    UseTarget::Name(name) => format!("name {name}"),
                    UseTarget::Member {
                        object, property, ..
                    } => format!("member {object} {property}"),
                    UseTarget::Method(name) => format!("method {name}"),
                    UseTarget::Load(name) => format!("load {name}"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn definition<'a>(index: &'a FileIndex) -> impl Fn(&str) -> Option<&'a Definition> {
    move |name| {
        index
            .definitions
            .iter()
            .find(|definition| definition.name == name)
    }
}

#[test]
fn methods_constructors_and_accessors_become_definitions() {
    let index = indexed(SOURCE);

    let names: Vec<&str> = index
        .definitions
        .iter()
        .map(|definition| definition.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "init",
            "Worker.constructor",
            "Worker.get_Name",
            "Worker.Run"
        ]
    );
    assert!(index.exports.contains(&Export::Local {
        exported: "Worker".to_string(),
        local: "Worker".to_string(),
    }));
    assert_eq!(
        targets_of(definition(&index)("init")),
        vec!["member HttpClient constructor", "member Worker Setup"]
    );
}

#[test]
fn calls_resolve_through_variable_types_this_and_type_names() {
    let index = indexed(SOURCE);

    assert_eq!(
        targets_of(definition(&index)("Worker.Run")),
        vec![
            "name init",
            "member Report constructor",
            "member Worker Helper",
            "member Worker Helper",
            "member Acme.Tools.Shell Run",
            "method Wait",
            "member Tools.Shell Execute",
            "member Report Save",
            "member Json SerializeObject",
            "member Worker Fetch",
        ]
    );
    assert_eq!(
        targets_of(definition(&index)("Worker.constructor")),
        vec!["name init", "member IRunner Go"]
    );
}

#[test]
fn type_names_bind_to_the_namespaces_in_scope() {
    let index = indexed(SOURCE);

    let binding = |specifier: &str, name: &str| Binding {
        specifier: specifier.to_string(),
        imported: Imported::Named(name.to_string()),
    };
    assert_eq!(
        index.bindings.get("Report"),
        Some(&binding("Report@Acme.App,Acme,System.Net.Http,", "Report"))
    );
    assert_eq!(
        index.bindings.get("Acme.Tools.Shell"),
        Some(&binding("Shell@Acme.Tools", "Shell"))
    );
    assert_eq!(
        index.bindings.get("Json"),
        Some(&binding("JsonConvert@Newtonsoft.Json", "JsonConvert"))
    );
    assert_eq!(index.bindings.get("Worker"), None);
    let loads: Vec<&str> = index
        .loads
        .iter()
        .map(|load| load.specifier.as_str())
        .collect();
    assert!(loads.contains(&"Shell@Acme.Tools"), "{loads:?}");
}

#[test]
fn a_skeleton_from_an_assembly_indexes_like_source() {
    let assembly = read(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/csharp/Acme.Tools.dll"
    )))
    .expect("assembly");
    let caller = assembly
        .types
        .iter()
        .find(|definition| definition.name == "Caller")
        .expect("caller");

    let index = indexed(&skeleton(caller));

    let targets = targets_of(definition(&index)("Caller.Call"));
    assert!(
        targets.contains(&"member IRunner Go".to_string()),
        "{targets:?}"
    );
    assert!(targets.contains(&"method Go".to_string()), "{targets:?}");
}
