use vigilante::Review;

use crate::support::{Branches, TestRepository, finding_lines};

const DEEP_PYTHON: &str = "[engines]\npython = \"deep\"\n";
const INTERNAL_CLIENT: &str = "id: internal-client\nlanguage: python\nmetadata:\n  capability: network\nrule:\n  pattern: internal_http.get($$$)\n";
const UNUSED_CLIENT: &str = "id: internal-client\nlanguage: python\nmetadata:\n  capability: network\nrule:\n  pattern: unused_http.get($$$)\n";
const COLLECTOR_CALL: &str = "internal_http.get(\"https://collector.example.invalid\")\n";

struct Change<'a> {
    base: &'a [(&'a str, &'a str)],
    head: &'a [(&'a str, &'a str)],
}

fn review_of(change: Change) -> Review {
    let repository = TestRepository::new();
    repository.write(change.base);
    let base = repository.commit("base");
    repository.write(change.head);
    let head = repository.commit("change");
    repository.diff(Branches {
        base: &base,
        head: &head,
    })
}

#[test]
fn a_pull_request_cannot_weaken_a_custom_rule_of_the_merge_base() {
    let config = format!(
        "{DEEP_PYTHON}[rules]\npaths = [\".vigilante/rules\"]\n[policy]\ndeny = [{{ capability = \"network\" }}]\n"
    );
    let review = review_of(Change {
        base: &[
            ("vigilante.toml", &config),
            (".vigilante/rules/internal.yml", INTERNAL_CLIENT),
        ],
        head: &[
            (".vigilante/rules/internal.yml", UNUSED_CLIENT),
            ("app.py", COLLECTOR_CALL),
        ],
    });

    assert_eq!(
        finding_lines(&review),
        vec!["code . network locked=false denied=true"]
    );
}
