use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde_json::Value;

const CONDITIONS: &[&str] = &["node", "require", "import", "module-sync", "default"];
const ROOT_SUBPATH: &str = ".";

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    main: Option<String>,
    #[serde(default)]
    exports: Option<Exports>,
    #[serde(default)]
    dependencies: HashMap<String, Value>,
    #[serde(default)]
    dev_dependencies: HashMap<String, Value>,
    #[serde(default)]
    optional_dependencies: HashMap<String, Value>,
    #[serde(default)]
    peer_dependencies: HashMap<String, Value>,
}

#[derive(Debug)]
enum Exports {
    Path(String),
    Conditions(Vec<(String, Exports)>),
    List(Vec<Exports>),
    Blocked,
}

impl Manifest {
    pub fn parse(text: &str) -> Self {
        serde_json::from_str(text).unwrap_or_default()
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn range_of(&self, name: &str) -> Option<String> {
        [
            &self.dependencies,
            &self.dev_dependencies,
            &self.optional_dependencies,
            &self.peer_dependencies,
        ]
        .iter()
        .find_map(|listed| listed.get(name))
        .map(|range| range.as_str().unwrap_or_default().to_string())
    }

    pub fn entries(&self, subpath: &str) -> Vec<String> {
        let wanted = if subpath.is_empty() {
            ROOT_SUBPATH.to_string()
        } else {
            format!("./{subpath}")
        };
        let exported = self
            .exports
            .as_ref()
            .map(|exports| exports.paths_for(&wanted))
            .unwrap_or_default();
        let mut entries: Vec<String> = exported.iter().map(|path| relative(path)).collect();
        if subpath.is_empty() {
            entries.push(
                self.main
                    .as_deref()
                    .map_or_else(|| "index".to_string(), relative),
            );
            entries.push("index".to_string());
        } else {
            entries.push(subpath.to_string());
        }
        entries
    }
}

impl Exports {
    fn paths_for(&self, wanted: &str) -> Vec<String> {
        let Self::Conditions(entries) = self else {
            return self.root_targets(wanted);
        };
        if !entries.iter().any(|(key, _)| key.starts_with('.')) {
            return self.root_targets(wanted);
        }
        if let Some((_, target)) = entries.iter().find(|(key, _)| key == wanted) {
            return target.targets(None);
        }
        entries
            .iter()
            .filter_map(|(key, target)| {
                let (prefix, suffix) = key.split_once('*')?;
                let middle = wanted.strip_prefix(prefix)?.strip_suffix(suffix)?;
                Some((prefix.len(), target.targets(Some(middle))))
            })
            .max_by_key(|(length, _)| *length)
            .map(|(_, paths)| paths)
            .unwrap_or_default()
    }

    fn root_targets(&self, wanted: &str) -> Vec<String> {
        if wanted == ROOT_SUBPATH {
            self.targets(None)
        } else {
            Vec::new()
        }
    }

    fn targets(&self, star: Option<&str>) -> Vec<String> {
        match self {
            Self::Path(path) => vec![match star {
                Some(star) => path.replace('*', star),
                None => path.clone(),
            }],
            Self::List(items) => items.iter().flat_map(|item| item.targets(star)).collect(),
            Self::Conditions(entries) => entries
                .iter()
                .filter(|(condition, _)| CONDITIONS.contains(&condition.as_str()))
                .flat_map(|(_, target)| target.targets(star))
                .collect(),
            Self::Blocked => Vec::new(),
        }
    }
}

fn relative(path: &str) -> String {
    path.trim_start_matches("./")
        .trim_end_matches('/')
        .to_string()
}

impl<'de> Deserialize<'de> for Exports {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ExportsVisitor)
    }
}

struct ExportsVisitor;

impl<'de> Visitor<'de> for ExportsVisitor {
    type Value = Exports;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a package.json exports value")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Exports, E> {
        Ok(Exports::Path(value.to_string()))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Exports, E> {
        Ok(Exports::Blocked)
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Exports, E> {
        Ok(Exports::Blocked)
    }

    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Exports, E> {
        Ok(Exports::Blocked)
    }

    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Exports, E> {
        Ok(Exports::Blocked)
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Exports, E> {
        Ok(Exports::Blocked)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Exports, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = sequence.next_element()? {
            items.push(item);
        }
        Ok(Exports::List(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Exports, A::Error> {
        let mut entries = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, Exports>()? {
            entries.push((key, value));
        }
        Ok(Exports::Conditions(entries))
    }
}
