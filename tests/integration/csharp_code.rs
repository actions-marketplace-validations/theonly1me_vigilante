use crate::csharp_fixtures::{Change, EXTRA_WITH_OLD_TOOLS, Feed, META, TOOLS, diff, project};
use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;

const APP_PROJECT: &str = "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>";
const LOCKFILE: &str = "{ \"version\": 1, \"dependencies\": { \"net8.0\": {} } }";

fn findings_after(change: Change) -> Vec<String> {
    let run = diff(change);
    summaries(&report_of(&run)["findings"])
}

fn reported_after(change: Change) -> (Vec<String>, Vec<String>) {
    let report = report_of(&diff(change));
    (summaries(&report["findings"]), summaries(&report["notes"]))
}

#[test]
fn own_code_that_reaches_exec_only_through_another_project_is_a_note() {
    let reported = reported_after(Change {
        base: &[
            ("App.csproj", APP_PROJECT),
            ("packages.lock.json", LOCKFILE),
            ("Tools/Tools.csproj", APP_PROJECT),
            ("Tools/packages.lock.json", LOCKFILE),
            (
                "Tools/A.cs",
                "namespace Acme.Quiet\n{\n    public static class Shell\n    {\n        public static void Run()\n        {\n        }\n    }\n}\n",
            ),
            (
                "Tools/Shell.cs",
                "using System.Diagnostics;\n\nnamespace Acme.Tools\n{\n    public static class Shell\n    {\n        public static void Run()\n        {\n            Process.Start(\"make\");\n        }\n    }\n}\n",
            ),
        ],
        head: &[(
            "Program.cs",
            "using Acme.Tools;\n\nnamespace App;\n\npublic class Program\n{\n    public static void Main()\n    {\n        Shell.Run();\n    }\n}\n",
        )],
    });

    assert_eq!(
        reported,
        (
            Vec::new(),
            vec![
                ". exec Tools Program.cs:9 Program.Program.Main > Tools/Shell.Shell.Run > Process.Start"
                    .to_string()
            ]
        )
    );
}

#[test]
fn a_method_reaches_the_network_through_a_field_initializer() {
    let reported = reported_after(Change {
        base: &[
            ("App.csproj", APP_PROJECT),
            ("packages.lock.json", LOCKFILE),
            ("Web/Web.csproj", APP_PROJECT),
            ("Web/packages.lock.json", LOCKFILE),
            (
                "Web/Fetcher.cs",
                "using System.Net.Http;\n\nnamespace Acme.Web;\n\npublic class Fetcher\n{\n    private readonly HttpClient client = new HttpClient();\n\n    public string Get()\n    {\n        return client.GetStringAsync(\"https://example.com\").Result;\n    }\n}\n",
            ),
        ],
        head: &[(
            "Program.cs",
            "using Acme.Web;\n\nnamespace App;\n\npublic class Program\n{\n    public static void Main()\n    {\n        var fetcher = new Fetcher();\n        fetcher.Get();\n    }\n}\n",
        )],
    });

    assert_eq!(
        reported,
        (
            Vec::new(),
            vec![
                ". network Web Program.cs:10 Program.Program.Main > Web/Fetcher.Fetcher.Get > Web/Fetcher.init > HttpClient"
                    .to_string()
            ]
        )
    );
}

#[test]
fn transitive_packages_are_resolved_and_the_nearest_version_wins() {
    let feed = Feed::serve(&[&META, &TOOLS, &EXTRA_WITH_OLD_TOOLS]);

    let run = diff(Change {
        base: &[
            ("App.csproj", &project(&[&META])),
            ("nuget.config", &feed.configuration()),
            ("vigilante.toml", &feed.allow_config()),
        ],
        head: &[(
            "Program.cs",
            "using Acme.Tools;\n\nnamespace App;\n\npublic class Program\n{\n    public static void Main()\n    {\n        Shell.Run(1);\n    }\n}\n",
        )],
    });

    let report = report_of(&run);
    let note = ". exec Acme.Tools Program.cs:9 Program.Program.Main > Acme.Tools/Acme.Tools/Shell.Shell.Run > Process.Start".to_string();
    assert_eq!(
        summaries(&report["notes"]),
        vec![note.clone(), note],
        "{}",
        run.stderr
    );
    assert_eq!(report["skipped"], serde_json::Value::Array(Vec::new()));
}

#[test]
fn a_test_project_folder_is_skipped_like_a_test_folder() {
    let findings = findings_after(Change {
        base: &[
            ("App.csproj", APP_PROJECT),
            ("packages.lock.json", LOCKFILE),
        ],
        head: &[
            ("App.Tests/App.Tests.csproj", APP_PROJECT),
            ("App.Tests/packages.lock.json", LOCKFILE),
            (
                "App.Tests/Support/HttpHelper.cs",
                "using System.Net.Http;\n\nnamespace App.Tests;\n\npublic class HttpHelper\n{\n    public string Get()\n    {\n        return new HttpClient().GetStringAsync(\"https://example.com\").Result;\n    }\n}\n",
            ),
        ],
    });

    assert_eq!(findings, Vec::<String>::new());
}
