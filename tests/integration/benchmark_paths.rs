use vigilante::Review;

use crate::support::{Branches, TestRepository, finding_lines};

const CONFIG: &str = "[engines]\njavascript = \"deep\"\n";
const MANIFEST: &str = "{\"name\": \"app\"}";
const APP: &str = "export function main() {}\n";
const SPAWN: &str = "import { execSync } from 'child_process';\nexecSync('make');\n";

fn review_of(path: &str) -> Review {
    let repository = TestRepository::new();
    repository.write(&[
        ("vigilante.toml", CONFIG),
        ("package.json", MANIFEST),
        ("src/app.js", APP),
    ]);
    let base = repository.commit("base");
    repository.write(&[(path, SPAWN)]);
    let head = repository.commit("change");
    repository.diff(Branches {
        base: &base,
        head: &head,
    })
}

fn test_evidence(review: &Review) -> Vec<String> {
    review
        .tests
        .iter()
        .map(|sighting| format!("{} {}", sighting.power, sighting.evidence.path))
        .collect()
}

#[test]
fn a_benchmark_folder_is_test_code() {
    for folder in ["bench", "benchmark", "benchmarks", "perf"] {
        let path = format!("{folder}/run.js");
        let review = review_of(&path);

        assert_eq!(finding_lines(&review), Vec::<String>::new(), "{folder}");
        assert_eq!(test_evidence(&review), vec![format!("exec {path}")]);
        assert!(review.passed(), "{folder}");
    }
}

#[test]
fn a_file_or_folder_that_only_contains_the_word_stays_production_code() {
    for path in ["src/perf.js", "perfect/run.js", "src/benchmarker/run.js"] {
        let review = review_of(path);

        assert_eq!(test_evidence(&review), Vec::<String>::new(), "{path}");
        assert_eq!(finding_lines(&review).len(), 1, "{path}");
    }
}
