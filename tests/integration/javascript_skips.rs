use crate::javascript_fixtures::{
    Change, RUNNING_MAIN, UNUSED_MAIN, diff, lock_entries, notes, served, skipped,
};
use crate::npm_registry::report_of;

#[test]
fn a_registry_that_is_not_allowed_is_listed_as_not_analyzed() {
    let registry = served();
    let lockfile = registry.lockfile(&lock_entries());

    let run = diff(Change {
        base_main: UNUSED_MAIN,
        head_main: RUNNING_MAIN,
        config: "",
        lockfile: Some(&lockfile),
    });

    let report = report_of(&run);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(notes(&report), Vec::<String>::new());
    assert_eq!(
        skipped(&report),
        vec![format!(
            "package-lock.json: `runner@1.0.0` comes from `{}`, which is not a public registry. Add it to `[registries] allow` in vigilante.toml if it needs no credentials.",
            registry.base_url
        )]
    );
}

#[test]
fn a_declared_package_that_is_not_in_the_lockfile_is_listed_as_not_analyzed() {
    let registry = served();
    let lockfile = registry.lockfile(&lock_entries());

    let run = diff(Change {
        base_main: UNUSED_MAIN,
        head_main: "import ghost from 'ghost';\nimport { toast } from '@acme/components/toast';\n\nexport function main() {\n  return ghost(toast);\n}\n",
        config: &registry.allow_config(),
        lockfile: Some(&lockfile),
    });

    assert_eq!(
        skipped(&report_of(&run)),
        vec!["package-lock.json: `ghost` is not in the lockfile."]
    );
}

#[test]
fn code_without_a_lockfile_is_listed_as_not_analyzed() {
    let run = diff(Change {
        base_main: UNUSED_MAIN,
        head_main: RUNNING_MAIN,
        config: "",
        lockfile: None,
    });

    assert_eq!(
        skipped(&report_of(&run)),
        vec![
            ".: No package-lock.json, npm-shrinkwrap.json, or pnpm-lock.yaml covers this code, so Vigilante did not read its packages."
        ]
    );
}
