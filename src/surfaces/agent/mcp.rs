use super::{Found, detection};
use crate::document::Node;
use crate::{Capability, Detection, Power};

const URL_KEYS: &[&str] = &["url", "httpUrl", "serverUrl"];

pub struct ServerSpec<'a> {
    pub name: &'a str,
    pub url: Option<&'a str>,
    pub command: Vec<&'a str>,
}

pub fn detect(servers: &Node) -> Vec<Detection> {
    servers
        .entries()
        .iter()
        .map(|entry| {
            let server = &entry.value;
            let spec = ServerSpec {
                name: &entry.key,
                url: URL_KEYS
                    .iter()
                    .find_map(|key| server.get(key).and_then(Node::text)),
                command: server
                    .get("command")
                    .and_then(Node::text)
                    .into_iter()
                    .chain(
                        server
                            .get("args")
                            .map(Node::items)
                            .unwrap_or_default()
                            .iter()
                            .filter_map(Node::text),
                    )
                    .collect(),
            };
            detection(Found {
                power: server_power(spec),
                line: entry.line,
            })
        })
        .collect()
}

pub fn server_power(spec: ServerSpec) -> Power {
    let target = match spec.url {
        Some(url) => url.to_string(),
        None => spec.command.join(" "),
    };
    let detail = if target.is_empty() {
        spec.name.to_string()
    } else {
        format!("{}: {target}", spec.name)
    };
    Power::new(Capability::McpServer).with_detail(detail)
}
