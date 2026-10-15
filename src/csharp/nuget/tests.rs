use super::frameworks::{FrameworkChoice, best_framework};
use super::lockfile::{LockedKind, read_lockfile};
use super::nuspec::{NuspecQuery, dependencies_for};
use super::versions::{lowest_version, normalized};

#[test]
fn versions_are_normalized_and_ranges_give_their_lowest_version() {
    assert_eq!(normalized("1.0"), "1.0.0");
    assert_eq!(normalized("4.7.0.0"), "4.7.0");
    assert_eq!(normalized("4.7.0.1"), "4.7.0.1");
    assert_eq!(normalized("2.0.0-Beta.1+build"), "2.0.0-beta.1");
    assert_eq!(lowest_version("13.0.3").as_deref(), Some("13.0.3"));
    assert_eq!(lowest_version("[1.2, 2.0)").as_deref(), Some("1.2.0"));
    assert_eq!(lowest_version("[3.1.1, )").as_deref(), Some("3.1.1"));
    assert_eq!(lowest_version("(, 2.0]"), None);
    assert_eq!(lowest_version("$(Missing)"), None);
}

#[test]
fn the_best_framework_is_the_closest_compatible_one() {
    let folders = [
        "net462",
        "netstandard2.0",
        "netstandard2.1",
        "net6.0",
        "net8.0-windows7.0",
    ];

    assert_eq!(
        best_framework(FrameworkChoice {
            project: "net8.0",
            candidates: &folders
        })
        .as_deref(),
        Some("net6.0")
    );
    assert_eq!(
        best_framework(FrameworkChoice {
            project: "net5.0",
            candidates: &folders
        })
        .as_deref(),
        Some("netstandard2.1")
    );
    assert_eq!(
        best_framework(FrameworkChoice {
            project: "net48",
            candidates: &folders
        })
        .as_deref(),
        Some("net462")
    );
    assert_eq!(
        best_framework(FrameworkChoice {
            project: "net8.0-windows",
            candidates: &folders
        })
        .as_deref(),
        Some("net8.0-windows7.0")
    );
    assert_eq!(
        best_framework(FrameworkChoice {
            project: "netstandard2.0",
            candidates: &["net6.0"]
        }),
        None
    );
    assert_eq!(
        best_framework(FrameworkChoice {
            project: "net8.0",
            candidates: &[".NETStandard2.0", ".NETFramework4.6.2"]
        })
        .as_deref(),
        Some(".NETStandard2.0")
    );
}

#[test]
fn a_lockfile_lists_the_packages_of_one_framework() {
    let text = r#"{
      "version": 1,
      "dependencies": {
        "net8.0": {
          "Serilog": { "type": "Direct", "requested": "[3.1.1, )", "resolved": "3.1.1", "contentHash": "abc=" },
          "System.Memory": { "type": "Transitive", "resolved": "4.5.5", "contentHash": "def=" },
          "Acme.Lib": { "type": "Project" }
        },
        "net8.0/linux-x64": {
          "Native.Only": { "type": "Transitive", "resolved": "1.0.0", "contentHash": "ghi=" }
        }
      }
    }"#;

    let locked = read_lockfile(text).expect("lockfile");

    let entries: Vec<(String, Option<String>, LockedKind)> = locked
        .iter()
        .map(|item| (item.id.clone(), item.version.clone(), item.kind))
        .collect();
    assert_eq!(
        entries,
        vec![
            ("Acme.Lib".to_string(), None, LockedKind::Project),
            (
                "Serilog".to_string(),
                Some("3.1.1".to_string()),
                LockedKind::Package
            ),
            (
                "System.Memory".to_string(),
                Some("4.5.5".to_string()),
                LockedKind::Package
            ),
        ]
    );
    assert_eq!(locked[1].content_hash.as_deref(), Some("abc="));
}

#[test]
fn a_nuspec_gives_the_dependencies_of_the_best_framework_group() {
    let text = r#"<?xml version="1.0"?><package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd"><metadata><id>Polly</id><version>8.4.0</version><dependencies>
      <group targetFramework=".NETFramework4.6.2"><dependency id="System.Old" version="1.0.0" /></group>
      <group targetFramework=".NETStandard2.0"><dependency id="Polly.Core" version="8.4.0" exclude="Build,Analyzers" /><dependency id="System.Memory" version="[4.5.5, )" /></group>
      <group targetFramework="net6.0"><dependency id="Polly.Core" version="8.4.0" /></group>
    </dependencies></metadata></package>"#;

    assert_eq!(
        dependencies_for(NuspecQuery {
            text,
            framework: "net8.0"
        }),
        vec![("Polly.Core".to_string(), "8.4.0".to_string())]
    );
    assert_eq!(
        dependencies_for(NuspecQuery {
            text,
            framework: "netstandard2.0"
        }),
        vec![
            ("Polly.Core".to_string(), "8.4.0".to_string()),
            ("System.Memory".to_string(), "[4.5.5, )".to_string()),
        ]
    );
}
