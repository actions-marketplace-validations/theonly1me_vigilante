use serde_json::Value;

use crate::Error;
use crate::registry::{Checksum, Registry, Sources};
use crate::xml::parse;

const NUGET_ORG_INDEX: &str = "https://api.nuget.org/v3/index.json";
const NUGET_ORG_PACKAGES: &str = "https://api.nuget.org/v3-flatcontainer/";
const NUGET_ORG_REGISTRATIONS: &str = "https://api.nuget.org/v3/registration5-gz-semver2/";
const PACKAGE_BASE: &str = "PackageBaseAddress/3.0.0";
const REGISTRATIONS: &str = "RegistrationsBaseUrl";
const SHA512_INTEGRITY: &str = "sha512-";

#[derive(Clone, Debug)]
pub struct Feed {
    pub packages: String,
    pub registrations: Option<String>,
}

pub struct Feeds {
    pub feeds: Vec<Feed>,
    pub not_public: Vec<String>,
}

pub struct FeedInput<'a> {
    pub registry: &'a Registry,
    pub sources: &'a Sources,
    pub config: Option<&'a str>,
}

pub fn feeds(input: FeedInput) -> Result<Feeds, Error> {
    let FeedInput {
        registry,
        sources,
        config,
    } = input;
    let mut addresses = vec![NUGET_ORG_INDEX.to_string()];
    if let Some(sources_element) = config
        .and_then(parse)
        .as_ref()
        .and_then(|configuration| configuration.child("packageSources"))
    {
        if sources_element.child("clear").is_some() {
            addresses.clear();
        }
        addresses.extend(
            sources_element
                .all("add")
                .into_iter()
                .filter_map(|source| source.attribute("value").map(str::to_string)),
        );
    }
    let mut found = Feeds {
        feeds: Vec::new(),
        not_public: Vec::new(),
    };
    for address in addresses {
        if address.starts_with("https://api.nuget.org/") {
            if !found
                .feeds
                .iter()
                .any(|feed| feed.packages == NUGET_ORG_PACKAGES)
            {
                found.feeds.push(Feed {
                    packages: NUGET_ORG_PACKAGES.to_string(),
                    registrations: Some(NUGET_ORG_REGISTRATIONS.to_string()),
                });
            }
            continue;
        }
        if !sources.allows(&address) {
            found.not_public.push(address);
            continue;
        }
        let Some(index) = registry.get_json(&address)? else {
            found.not_public.push(address);
            continue;
        };
        let resource = |kind: &str| {
            index
                .get("resources")
                .and_then(Value::as_array)?
                .iter()
                .find(|resource| {
                    resource
                        .get("@type")
                        .and_then(Value::as_str)
                        .is_some_and(|value| value.starts_with(kind))
                })
                .and_then(|resource| resource.get("@id"))
                .and_then(Value::as_str)
                .map(|address| format!("{}/", address.trim_end_matches('/')))
        };
        match resource(PACKAGE_BASE) {
            Some(packages) if sources.allows(&packages) => found.feeds.push(Feed {
                packages,
                registrations: resource(REGISTRATIONS).filter(|address| sources.allows(address)),
            }),
            _ => found.not_public.push(address),
        }
    }
    Ok(found)
}

pub struct PackageAddress<'a> {
    pub feed: &'a Feed,
    pub id: &'a str,
    pub version: &'a str,
}

impl PackageAddress<'_> {
    pub fn file(&self, extension: &str) -> String {
        let id = self.id.to_lowercase();
        let version = self.version.to_lowercase();
        match extension {
            "nuspec" => format!("{}{id}/{version}/{id}.nuspec", self.feed.packages),
            _ => format!(
                "{}{id}/{version}/{id}.{version}.{extension}",
                self.feed.packages
            ),
        }
    }

    pub fn checksum(&self, registry: &Registry) -> Result<Option<Checksum>, Error> {
        let Some(registrations) = &self.feed.registrations else {
            return Ok(None);
        };
        let leaf = format!(
            "{registrations}{}/{}.json",
            self.id.to_lowercase(),
            self.version.to_lowercase()
        );
        let Some(leaf) = registry.get_json(&leaf)? else {
            return Ok(None);
        };
        let catalog = match leaf.get("catalogEntry") {
            Some(Value::String(address)) => Some(address.clone()),
            Some(entry) => entry.get("@id").and_then(Value::as_str).map(str::to_string),
            None => None,
        };
        let Some(catalog) = catalog else {
            return Ok(None);
        };
        Ok(registry
            .get_json(&catalog)?
            .and_then(|entry| {
                entry
                    .get("packageHash")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .and_then(|hash| content_checksum(&hash)))
    }
}

pub fn content_checksum(hash: &str) -> Option<Checksum> {
    Checksum::from_integrity(&format!("{SHA512_INTEGRITY}{hash}"))
}
