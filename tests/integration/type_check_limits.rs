use crate::slow_tools::{
    FAST_ENOUGH, Project, RUST_BASE, RUST_HEAD, RUST_OUT_OF_TIME, SLOW_DOCKER, SLOW_NODE, reasons,
    review_with_slow_tool,
};

#[test]
#[ignore = "needs network"]
fn a_typescript_check_that_reaches_the_time_limit_falls_back_to_names() {
    let timed = review_with_slow_tool(Project {
        engine: "javascript",
        base: &[
            ("package.json", "{\"name\": \"app\"}"),
            ("src/index.ts", "export function main() {\n  return 0;\n}\n"),
        ],
        head: (
            "src/index.ts",
            "import { execSync } from 'child_process';\n\nexport function main() {\n  return execSync('make');\n}\n",
        ),
        tool: SLOW_NODE,
    });

    assert!(
        reasons(&timed.report).contains(
            &"The TypeScript checker reached the time limit of type_check_seconds = 1, so Vigilante resolved calls by name only.".to_string()
        ),
        "{}",
        timed.report
    );
    assert!(timed.elapsed < FAST_ENOUGH, "{:?}", timed.elapsed);
}

#[test]
#[ignore = "needs network"]
fn a_python_check_that_reaches_the_time_limit_falls_back_to_names() {
    let timed = review_with_slow_tool(Project {
        engine: "python",
        base: &[
            (
                "pyproject.toml",
                "[project]\nname = \"app\"\nversion = \"0.1.0\"\n",
            ),
            ("app.py", "def main():\n    return 0\n"),
        ],
        head: (
            "app.py",
            "import subprocess\n\n\ndef main():\n    return subprocess.run(['make'])\n",
        ),
        tool: SLOW_NODE,
    });

    assert!(
        reasons(&timed.report).contains(
            &"scip-python reached the time limit of type_check_seconds = 1, so Vigilante resolved Python calls by name only.".to_string()
        ),
        "{}",
        timed.report
    );
    assert!(timed.elapsed < FAST_ENOUGH, "{:?}", timed.elapsed);
}

#[test]
#[ignore = "needs docker"]
fn docker_a_rust_build_that_reaches_the_time_limit_falls_back_to_names() {
    let timed = review_with_slow_tool(Project {
        engine: "rust",
        base: RUST_BASE,
        head: RUST_HEAD,
        tool: SLOW_NODE,
    });

    assert!(
        reasons(&timed.report).contains(&RUST_OUT_OF_TIME.to_string()),
        "{}",
        timed.report
    );
    assert!(timed.elapsed < FAST_ENOUGH, "{:?}", timed.elapsed);
}

#[test]
fn a_rust_image_download_that_reaches_the_time_limit_falls_back_to_names() {
    let timed = review_with_slow_tool(Project {
        engine: "rust",
        base: RUST_BASE,
        head: RUST_HEAD,
        tool: SLOW_DOCKER,
    });

    assert!(
        reasons(&timed.report).contains(&RUST_OUT_OF_TIME.to_string()),
        "{}",
        timed.report
    );
    assert!(timed.elapsed < FAST_ENOUGH, "{:?}", timed.elapsed);
}
