use std::env;
use std::path::PathBuf;

const NODE: &str = "node";

pub fn node_program() -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|folder| folder.join(NODE))
        .find(|candidate| candidate.is_file())
}
