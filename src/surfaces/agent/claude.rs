use super::{Found, allow_rule, detection};
use crate::document::Node;
use crate::{Capability, Detection, Power};

const COMMAND_SETTINGS: &[&str] = &["apiKeyHelper", "awsAuthRefresh", "awsCredentialExport"];

pub fn detect(settings: &Node) -> Vec<Detection> {
    let mut found = Vec::new();
    let permissions = settings.get("permissions");
    for rule in permissions
        .and_then(|permissions| permissions.get("allow"))
        .map(Node::items)
        .unwrap_or_default()
    {
        if let Some(text) = rule.text()
            && allow_rule::grants_power(text)
        {
            found.push(Found {
                power: Power::new(Capability::ToolAllow).with_detail(text),
                line: rule.line,
            });
        }
    }
    if let Some(mode) = permissions.and_then(|permissions| permissions.get("defaultMode"))
        && mode.text() == Some("bypassPermissions")
    {
        found.push(Found {
            power: Power::new(Capability::SkipPermissions).with_detail("bypassPermissions"),
            line: mode.line,
        });
    }
    if let Some(flag) = settings.get("enableAllProjectMcpServers")
        && flag.text() == Some("true")
    {
        found.push(Found {
            power: Power::new(Capability::SkipPermissions)
                .with_detail("enableAllProjectMcpServers"),
            line: flag.line,
        });
    }
    for server in settings
        .get("enabledMcpjsonServers")
        .map(Node::items)
        .unwrap_or_default()
    {
        if let Some(name) = server.text() {
            found.push(Found {
                power: Power::new(Capability::SkipPermissions)
                    .with_detail(format!("enabledMcpjsonServers: {name}")),
                line: server.line,
            });
        }
    }
    found.extend(hooks(settings));
    found.into_iter().map(detection).collect()
}

fn hooks(settings: &Node) -> Vec<Found> {
    let mut found = Vec::new();
    for event in settings.get("hooks").map(Node::entries).unwrap_or_default() {
        for group in event.value.items() {
            for hook in group.get("hooks").map(Node::items).unwrap_or_default() {
                if let Some(command) = hook.get("command").and_then(Node::text) {
                    found.push(command_hook(CommandHook {
                        name: &event.key,
                        command,
                        line: hook.line,
                    }));
                }
            }
        }
    }
    if let Some(status_line) = settings.get("statusLine")
        && let Some(command) = status_line.get("command").and_then(Node::text)
    {
        found.push(command_hook(CommandHook {
            name: "statusLine",
            command,
            line: status_line.line,
        }));
    }
    for name in COMMAND_SETTINGS {
        if let Some(setting) = settings.get(name)
            && let Some(command) = setting.text()
        {
            found.push(command_hook(CommandHook {
                name,
                command,
                line: setting.line,
            }));
        }
    }
    found
}

struct CommandHook<'a> {
    name: &'a str,
    command: &'a str,
    line: usize,
}

fn command_hook(hook: CommandHook) -> Found {
    Found {
        power: Power::new(Capability::Hook).with_detail(format!("{}: {}", hook.name, hook.command)),
        line: hook.line,
    }
}
