use super::capslock::CapabilityInfo;
use crate::{Capability, Error, Evidence, Power};

const RULE: &str = "capslock";
const CAPSLOCK_KINDS: &[(&str, Capability, Option<&str>)] = &[
    ("ARBITRARY_EXECUTION", Capability::ArbitraryExecution, None),
    ("CGO", Capability::Native, Some("cgo")),
    ("EXEC", Capability::Exec, None),
    ("FILES", Capability::Files, None),
    ("MODIFY_SYSTEM_STATE", Capability::ModifySystemState, None),
    ("NETWORK", Capability::Network, None),
    ("OPERATING_SYSTEM", Capability::OperatingSystem, None),
    ("READ_SYSTEM_STATE", Capability::ReadSystemState, None),
    ("REFLECT", Capability::DynamicCode, Some("reflect")),
    ("RUNTIME", Capability::Runtime, None),
    ("SYSTEM_CALLS", Capability::SystemCalls, None),
    ("UNANALYZED", Capability::Unanalyzed, None),
    ("UNSAFE_POINTER", Capability::Native, Some("unsafe_pointer")),
];

#[derive(Clone, Debug)]
pub struct Module {
    pub folder: String,
    pub path: String,
}

pub fn power_of(capslock_name: &str) -> Result<Power, Error> {
    let (kind, sub_kind) = match capslock_name.split_once('/') {
        Some((kind, sub_kind)) => (kind, Some(sub_kind.to_lowercase())),
        None => (capslock_name, None),
    };
    let (_, capability, fixed_detail) = CAPSLOCK_KINDS
        .iter()
        .find(|(name, ..)| *name == kind)
        .ok_or_else(|| {
            Error::new(format!(
                "Capslock reported the capability `{capslock_name}`, which this Vigilante version does not know. Use Capslock v0.3.3 or upgrade Vigilante."
            ))
        })?;
    let power = Power::new(*capability);
    Ok(match fixed_detail.map(str::to_string).or(sub_kind) {
        Some(detail) => power.with_detail(detail),
        None => power,
    })
}

pub struct PackageLookup<'a> {
    pub package: &'a str,
    pub modules: &'a [Module],
}

pub fn folder_of(lookup: PackageLookup) -> Option<String> {
    let PackageLookup { package, modules } = lookup;
    modules
        .iter()
        .filter_map(|module| {
            let rest = if package == module.path {
                ""
            } else {
                package.strip_prefix(&module.path)?.strip_prefix('/')?
            };
            Some((module, rest))
        })
        .max_by_key(|(module, _)| module.path.len())
        .map(|(module, rest)| {
            joined(InFolder {
                folder: &module.folder,
                rest,
            })
        })
}

pub struct EvidenceStart<'a> {
    pub info: &'a CapabilityInfo,
    pub package: &'a str,
    pub folder: &'a str,
}

pub fn evidence_of(start: EvidenceStart) -> Evidence {
    let EvidenceStart {
        info,
        package,
        folder,
    } = start;
    let site = info
        .path
        .iter()
        .position(|function| function.package == package)
        .and_then(|index| info.path.get(index + 1))
        .and_then(|function| function.site.as_ref());
    let (path, line) = match site {
        Some(site) => (
            joined(InFolder {
                folder,
                rest: &file_name(&site.filename),
            }),
            site.line.parse().unwrap_or(1),
        ),
        None => (folder.to_string(), 1),
    };
    Evidence {
        path,
        line,
        rule: RULE.to_string(),
        call_path: info
            .path
            .iter()
            .map(|function| function.name.clone())
            .collect(),
    }
}

pub struct InFolder<'a> {
    pub folder: &'a str,
    pub rest: &'a str,
}

pub fn joined(path: InFolder) -> String {
    let InFolder { folder, rest } = path;
    match (folder, rest) {
        (_, "") => folder.to_string(),
        (".", _) => rest.to_string(),
        _ => format!("{folder}/{rest}"),
    }
}

fn file_name(reported: &str) -> String {
    reported
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(reported)
        .chars()
        .filter(|character| !character.is_control())
        .collect()
}
