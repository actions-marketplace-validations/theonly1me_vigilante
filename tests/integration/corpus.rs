use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use serde::Deserialize;
use vigilante::Review;

use crate::support::{Branches, TestRepository};

const ENGINES: &str = "[engines]\npython = \"deep\"\njavascript = \"deep\"\n";
const CORPUS: [&str; 2] = [
    include_str!("../corpus/code.toml"),
    include_str!("../corpus/surfaces.toml"),
];

type Files = BTreeMap<String, String>;

#[derive(Deserialize)]
struct Corpus {
    case: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    group: String,
    base: Files,
    #[serde(rename = "unsafe")]
    unsafe_change: Files,
    #[serde(rename = "safe")]
    safe_change: Files,
    #[serde(default = "blocks")]
    unsafe_result: Outcome,
    reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Blocks,
    Shows,
    Passes,
}

fn blocks() -> Outcome {
    Outcome::Blocks
}

#[derive(Default)]
struct Tally {
    pairs: usize,
    unsafe_caught: usize,
    unsafe_blocked: usize,
    safe_blocked: usize,
    safe_shown: usize,
    slowest: Duration,
}

struct Change<'a> {
    base: &'a Files,
    head: &'a Files,
}

struct Measured {
    outcome: Outcome,
    time: Duration,
}

fn measure(change: Change) -> Measured {
    let repository = TestRepository::new();
    let files_of = |files: &'_ Files| -> Vec<(String, String)> {
        files
            .iter()
            .map(|(path, text)| (path.clone(), text.trim_start().to_string()))
            .collect()
    };
    let written = |files: &[(String, String)]| {
        let pairs: Vec<(&str, &str)> = files
            .iter()
            .map(|(path, text)| (path.as_str(), text.as_str()))
            .collect();
        repository.write(&pairs);
    };
    repository.write(&[("vigilante.toml", ENGINES)]);
    written(&files_of(change.base));
    let base = repository.commit("base");
    written(&files_of(change.head));
    let head = repository.commit("change");
    let started = Instant::now();
    let review = repository.diff(Branches {
        base: &base,
        head: &head,
    });
    Measured {
        outcome: outcome_of(&review),
        time: started.elapsed(),
    }
}

fn outcome_of(review: &Review) -> Outcome {
    let shows = !(review.findings.is_empty() && review.risks.is_empty() && review.notes.is_empty());
    match (review.passed(), shows) {
        (false, _) => Outcome::Blocks,
        (true, true) => Outcome::Shows,
        (true, false) => Outcome::Passes,
    }
}

fn table(groups: &BTreeMap<String, Tally>) -> String {
    let mut lines = vec![
        "| Group | Pairs | Unsafe changes caught | Unsafe changes that block | Safe changes that block | Safe changes shown | Slowest review |".to_string(),
        "| --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];
    lines.extend(groups.iter().map(|(group, tally)| {
        format!(
            "| {group} | {} | {} | {} | {} | {} | {} ms |",
            tally.pairs,
            tally.unsafe_caught,
            tally.unsafe_blocked,
            tally.safe_blocked,
            tally.safe_shown,
            tally.slowest.as_millis()
        )
    }));
    lines.join("\n")
}

#[test]
fn each_unsafe_change_is_caught_and_each_safe_change_passes() {
    run_corpus(&CORPUS);
}

#[test]
#[ignore = "needs go and capslock on PATH"]
fn each_unsafe_go_change_is_caught_and_each_safe_go_change_passes() {
    run_corpus(&[include_str!("../corpus/go.toml")]);
}

fn run_corpus(texts: &[&str]) {
    let cases = texts
        .iter()
        .flat_map(|text| toml::from_str::<Corpus>(text).expect("corpus").case);
    let mut mismatches = Vec::new();
    let mut groups: BTreeMap<String, Tally> = BTreeMap::new();
    for case in cases {
        let unsafe_run = measure(Change {
            base: &case.base,
            head: &case.unsafe_change,
        });
        let safe_run = measure(Change {
            base: &case.base,
            head: &case.safe_change,
        });
        if unsafe_run.outcome != case.unsafe_result {
            mismatches.push(format!(
                "{}: the unsafe change {:?}, expected {:?}",
                case.name, unsafe_run.outcome, case.unsafe_result
            ));
        }
        if safe_run.outcome != Outcome::Passes {
            mismatches.push(format!(
                "{}: the safe change {:?}",
                case.name, safe_run.outcome
            ));
        }
        if case.unsafe_result != Outcome::Blocks && case.reason.is_none() {
            mismatches.push(format!(
                "{}: a case that does not block names its reason",
                case.name
            ));
        }
        let tally = groups.entry(case.group).or_default();
        tally.pairs += 1;
        tally.unsafe_caught += usize::from(unsafe_run.outcome != Outcome::Passes);
        tally.unsafe_blocked += usize::from(unsafe_run.outcome == Outcome::Blocks);
        tally.safe_blocked += usize::from(safe_run.outcome == Outcome::Blocks);
        tally.safe_shown += usize::from(safe_run.outcome == Outcome::Shows);
        tally.slowest = tally.slowest.max(unsafe_run.time).max(safe_run.time);
    }
    println!("{}", table(&groups));
    assert_eq!(mismatches, Vec::<String>::new());
}
