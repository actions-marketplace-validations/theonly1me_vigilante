use crate::code::Rules;
use crate::config::Config;
use crate::tree::Tree;
use crate::{Error, SourceFile};

pub struct RuleSources<'a> {
    pub tree: &'a dyn Tree,
    pub config: &'a Config,
}

pub fn rules_of(sources: RuleSources) -> Result<Rules, Error> {
    let mut rules = Rules::built_in()?;
    let folders: Vec<String> = sources
        .config
        .rule_paths
        .iter()
        .map(|folder| format!("{}/", folder.trim_end_matches('/')))
        .collect();
    let rule_files: Vec<&str> = sources
        .tree
        .paths()
        .iter()
        .map(String::as_str)
        .filter(|path| {
            folders
                .iter()
                .any(|folder| path.starts_with(folder.as_str()))
                && (path.ends_with(".yml") || path.ends_with(".yaml"))
        })
        .collect();
    for file in sources.tree.read(&rule_files)? {
        rules.add(SourceFile {
            path: &file.path,
            text: &file.text,
        })?;
    }
    Ok(rules)
}
