use toml::{Table, Value};

use super::mcp::{ServerSpec, server_power};
use super::{Found, detection};
use crate::{Capability, Detection, Error, Power, SourceFile};

const RISKY_SETTINGS: &[(&str, &str)] = &[
    ("approval_policy", "never"),
    ("sandbox_mode", "danger-full-access"),
];

pub fn detect(file: SourceFile) -> Result<Vec<Detection>, Error> {
    let config: Table = file
        .text
        .parse()
        .map_err(|error| Error::new(format!("cannot parse {}: {error}", file.path)))?;
    let mut found = Vec::new();
    for (key, risky_value) in RISKY_SETTINGS {
        if config.get(*key).and_then(Value::as_str) == Some(*risky_value) {
            found.push(Found {
                power: Power::new(Capability::SkipPermissions)
                    .with_detail(format!("{key}={risky_value}")),
                line: line_of(Needle {
                    text: file.text,
                    needle: key,
                }),
            });
        }
    }
    let servers = config.get("mcp_servers").and_then(Value::as_table);
    for (name, server) in servers.into_iter().flatten() {
        let text_of = |key: &str| server.get(key).and_then(Value::as_str);
        let arguments = server
            .get("args")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let spec = ServerSpec {
            name,
            url: text_of("url"),
            command: text_of("command")
                .into_iter()
                .chain(arguments.iter().filter_map(Value::as_str))
                .collect(),
        };
        found.push(Found {
            power: server_power(spec),
            line: line_of(Needle {
                text: file.text,
                needle: &format!("mcp_servers.{name}"),
            }),
        });
    }
    Ok(found.into_iter().map(detection).collect())
}

struct Needle<'a> {
    text: &'a str,
    needle: &'a str,
}

fn line_of(search: Needle) -> usize {
    search
        .text
        .lines()
        .position(|line| line.contains(search.needle))
        .map_or(1, |index| index + 1)
}
