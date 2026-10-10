use std::collections::BTreeSet;

use crate::go_fixtures::{
    Call, DIFF, FETCH_PACKAGE, PAGE, Reach, capslock_output, direct_exec_reaches,
    finding_summaries, module_repository, network_reaches,
};
use crate::go_notes::{diff, note_summaries};
use crate::go_support::report_of;

const SIGNAL_NOTIFY: Call = Call {
    function: "os/signal.Notify",
    package: "os/signal",
    site: Some(("fetch.go", 5)),
};

#[test]
fn sarif_and_json_carry_the_call_path() {
    let repository = module_repository(&network_reaches());
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let both: Vec<Reach> = network_reaches()
        .into_iter()
        .chain(direct_exec_reaches())
        .collect();
    repository.write(&[("app/capslock.json", &capslock_output(&both))]);
    repository.commit("run a tool");

    let json = report_of(&diff(&repository, DIFF));
    let sarif = report_of(&diff(
        &repository,
        &[
            "diff", "--base", "main", "--head", "feature", "--format", "sarif",
        ],
    ));

    assert_eq!(
        json["findings"][0]["evidence"]["call_path"],
        serde_json::json!(["example.com/app/fetch.Page", "os/exec.Command"])
    );
    let message = sarif["runs"][0]["results"][0]["message"]["text"].to_string();
    assert!(message.contains("fetch.Page → exec.Command"), "{message}");
}

#[test]
fn only_exec_network_and_native_block_and_other_go_powers_are_notes() {
    let repository = module_repository(&network_reaches());
    repository.commit("base");
    repository.git(&["checkout", "--quiet", "-b", "feature"]);
    let mut head = network_reaches();
    head.push(Reach {
        package: FETCH_PACKAGE,
        capability: "MODIFY_SYSTEM_STATE/SIGNALS",
        calls: &[PAGE, SIGNAL_NOTIFY],
    });
    repository.write(&[("app/capslock.json", &capslock_output(&head))]);
    repository.commit("handle signals");

    let run = diff(&repository, DIFF);

    let report = report_of(&run);
    assert!(finding_summaries(&report).is_empty(), "{}", run.stdout);
    assert_eq!(
        note_summaries(&report),
        BTreeSet::from([
            "app/fetch modify_system_state:signals via os/signal app/fetch/fetch.go:5".to_string()
        ])
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
}
