use vigilante::Review;

use crate::support::{Branches, TestRepository};

const NAME_BASED: &str = "[engines]\njavascript = \"deep\"\n";
const MANIFEST: &str = "{\"name\": \"app\", \"version\": \"1.0.0\"}\n";

fn review_with_head(head: &str) -> Review {
    let repository = TestRepository::new();
    repository.write(&[
        ("vigilante.toml", NAME_BASED),
        ("package.json", MANIFEST),
        ("index.js", "exports.run = () => 0;\n"),
    ]);
    let base = repository.commit("base");
    repository.write(&[("index.js", head)]);
    let head = repository.commit("change");
    repository.diff(Branches {
        base: &base,
        head: &head,
    })
}

fn gap_lines(review: &Review) -> Vec<String> {
    review
        .new_gaps
        .iter()
        .map(|gap| format!("{}: {}", gap.source, gap.reason))
        .collect()
}

#[test]
fn a_require_through_a_constant_module_name_is_an_analysis_gap() {
    let review = review_with_head(
        "const name = \"node:child_process\";\nconst processes = require(name);\nexports.run = (command) => processes.exec(command);\n",
    );

    assert!(
        gap_lines(&review).contains(&"index.js: Line 2 loads a module by a name that Vigilante cannot read, so Vigilante did not follow it.".to_string()),
        "{:?}",
        gap_lines(&review)
    );
}

#[test]
fn a_load_with_a_module_name_that_vigilante_cannot_read_is_an_analysis_gap() {
    let review = review_with_head("exports.run = (name) => require(name).exec(\"ls\");\n");

    let gaps = gap_lines(&review);
    assert!(
        gaps.contains(&"index.js: Line 1 loads a module by a name that Vigilante cannot read, so Vigilante did not follow it.".to_string()),
        "{gaps:?}"
    );
}
