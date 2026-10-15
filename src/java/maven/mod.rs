mod effective;
mod pom;
mod properties;
mod resolve;
#[cfg(test)]
mod tests;

pub use effective::{EffectiveRequest, PomSource, effective};
pub use pom::{Coordinates, Pom, parse_pom};
pub use resolve::{ResolveRequest, resolve};
