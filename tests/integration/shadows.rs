use crate::support::{TestRepository, finding_lines};

const NAME_BASED: &str = "[engines]\njavascript = \"deep\"\npython = \"deep\"\n";
const NETWORK: &str = "code . network locked=false denied=false";

fn findings_of(files: &[(&str, &str)]) -> Vec<String> {
    let repository = TestRepository::new();
    repository.write(&[("vigilante.toml", NAME_BASED)]);
    repository.write(files);
    repository.commit("code");
    finding_lines(&repository.check())
}

#[test]
fn the_fetch_handler_of_a_worker_does_not_hide_a_global_fetch() {
    let worker = "async function store(url) {\n  return fetch(url);\n}\n\nexport default {\n  async fetch(request, env) {\n    return store(env.URL);\n  },\n};\n";

    assert_eq!(findings_of(&[("worker.js", worker)]), vec![NETWORK]);
}

#[test]
fn a_class_method_named_fetch_does_not_hide_a_global_fetch() {
    let client = "class Client {\n  fetch(url) {\n    return url;\n  }\n}\n\nexport function send(url) {\n  return fetch(url);\n}\n";

    assert_eq!(findings_of(&[("client.js", client)]), vec![NETWORK]);
}

#[test]
fn a_python_method_named_eval_does_not_hide_the_eval_builtin() {
    let calculator = "class Calculator:\n    def eval(self, text):\n        return text\n\n\ndef run(request):\n    return eval(request.args[\"expression\"])\n";
    let repository = TestRepository::new();
    repository.write(&[
        ("vigilante.toml", NAME_BASED),
        ("calculator.py", calculator),
    ]);
    repository.commit("code");

    let risks: Vec<String> = repository
        .check()
        .risks
        .iter()
        .map(|risk| risk.identity.clone())
        .collect();

    assert_eq!(risks, vec!["dynamic_code:calculator.run:eval:untrusted"]);
}
