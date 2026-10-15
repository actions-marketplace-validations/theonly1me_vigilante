mod lockfile;
#[cfg(test)]
mod tests;

pub use lockfile::{GemSource, LockedGem, read_lockfile};
