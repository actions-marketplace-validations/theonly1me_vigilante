const CODE_EXTENSIONS: &[&str] = &[".js", ".ts", ".tsx", ".mjs", ".cjs", ".jsx", ".mts", ".cts"];
const TYPESCRIPT_SWAPS: &[(&str, &[&str])] = &[
    (".js", &[".ts", ".tsx"]),
    (".jsx", &[".tsx"]),
    (".mjs", &[".mts"]),
    (".cjs", &[".cts"]),
];
const NODE_BUILT_INS: &[&str] = &[
    "assert",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "sea",
    "sqlite",
    "stream",
    "string_decoder",
    "sys",
    "test",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

pub fn is_code_file(path: &str) -> bool {
    CODE_EXTENSIONS
        .iter()
        .any(|extension| path.ends_with(extension))
        && ![".d.ts", ".d.mts", ".d.cts"]
            .iter()
            .any(|declaration| path.ends_with(declaration))
}

pub fn is_built_in(specifier: &str) -> bool {
    if specifier.starts_with("node:") || specifier.starts_with("bun:") {
        return true;
    }
    let first = specifier.split('/').next().unwrap_or(specifier);
    NODE_BUILT_INS.contains(&first)
}

pub fn is_relative(specifier: &str) -> bool {
    specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
}

pub struct PackageSpecifier<'a> {
    pub name: &'a str,
    pub subpath: &'a str,
}

pub fn package_specifier(specifier: &str) -> Option<PackageSpecifier<'_>> {
    let split_at = if specifier.starts_with('@') {
        let scope_end = specifier.find('/')?;
        specifier[scope_end + 1..]
            .find('/')
            .map(|index| scope_end + 1 + index)
    } else {
        specifier.find('/')
    };
    let (name, subpath) = match split_at {
        Some(index) => (&specifier[..index], &specifier[index + 1..]),
        None => (specifier, ""),
    };
    is_package_name(name).then_some(PackageSpecifier { name, subpath })
}

fn is_package_name(name: &str) -> bool {
    let plain = |part: &str| {
        !part.is_empty()
            && !part.starts_with('.')
            && !part.starts_with('_')
            && part.chars().all(|character| {
                character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || "-._~".contains(character)
            })
    };
    match name.strip_prefix('@') {
        Some(scoped) => scoped
            .split_once('/')
            .is_some_and(|(scope, base)| plain(scope) && plain(base)),
        None => plain(name),
    }
}

pub fn parent_folder(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(folder, _)| folder)
}

pub fn joined(folder: &str, relative: &str) -> Option<String> {
    let mut parts: Vec<&str> = folder
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect();
    for part in relative.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

pub fn probe(exists: &dyn Fn(&str) -> bool, base: &str) -> Option<String> {
    let base = base.trim_end_matches('/');
    let mut candidates = Vec::new();
    if is_code_file(base) {
        candidates.push(base.to_string());
    }
    for (written, swaps) in TYPESCRIPT_SWAPS {
        if let Some(stem) = base.strip_suffix(written) {
            candidates.extend(swaps.iter().map(|swap| format!("{stem}{swap}")));
        }
    }
    candidates.extend(
        CODE_EXTENSIONS
            .iter()
            .map(|extension| format!("{base}{extension}")),
    );
    let index = if base.is_empty() {
        "index".to_string()
    } else {
        format!("{base}/index")
    };
    candidates.extend(
        CODE_EXTENSIONS
            .iter()
            .map(|extension| format!("{index}{extension}")),
    );
    candidates.into_iter().find(|candidate| exists(candidate))
}

pub fn install_candidates(folder: &str, name: &str) -> Vec<String> {
    let parts: Vec<&str> = folder.split('/').filter(|part| !part.is_empty()).collect();
    let mut candidates = Vec::new();
    for length in (0..=parts.len()).rev() {
        let prefix = &parts[..length];
        if prefix.last() == Some(&"node_modules") {
            continue;
        }
        candidates.push(match prefix {
            [] => format!("node_modules/{name}"),
            _ => format!("{}/node_modules/{name}", prefix.join("/")),
        });
    }
    candidates
}

pub fn stem(path: &str) -> &str {
    CODE_EXTENSIONS
        .iter()
        .find_map(|extension| path.strip_suffix(extension))
        .unwrap_or(path)
}
