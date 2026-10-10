const STANDARD_LIBRARY: &str = include_str!("standard_library.txt");

pub fn is_standard(module: &str) -> bool {
    let top = module.split('.').next().unwrap_or(module);
    STANDARD_LIBRARY.lines().any(|name| name == top)
}
