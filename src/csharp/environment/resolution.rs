use std::collections::HashMap;

use super::feeds::{Feed, PackageAddress};
use super::parallel::{ParallelWork, each_in_parallel};
use crate::Error;
use crate::csharp::nuget::nuspec::{NuspecQuery, dependencies_for};
use crate::csharp::nuget::versions::{VersionPair, is_newer, lowest_version};
use crate::registry::Registry;

const MAXIMUM_DEPTH: usize = 32;

pub struct ResolveInput<'a> {
    pub references: &'a [(String, String)],
    pub framework: &'a str,
    pub feeds: &'a [Feed],
    pub registry: &'a Registry,
}

#[derive(Clone, Debug)]
pub struct Resolved {
    pub id: String,
    pub version: String,
    pub feed: Option<usize>,
}

pub struct Resolution {
    pub packages: Vec<Resolved>,
    pub skipped: Vec<String>,
}

pub fn resolve(input: ResolveInput) -> Result<Resolution, Error> {
    let ResolveInput {
        references,
        framework,
        feeds,
        registry,
    } = input;
    let mut skipped = Vec::new();
    let mut chosen: HashMap<String, Resolved> = HashMap::new();
    let mut level: Vec<(String, String)> = Vec::new();
    for (id, range) in references {
        match lowest_version(range) {
            Some(version) => level.push((id.clone(), version)),
            None => skipped.push(format!(
                "`{id}` has no exact version, so Vigilante did not read it."
            )),
        }
    }
    for _ in 0..MAXIMUM_DEPTH {
        let mut fresh: HashMap<String, (String, String)> = HashMap::new();
        for (id, version) in level {
            let key = id.to_lowercase();
            if chosen.contains_key(&key) {
                continue;
            }
            let replace = fresh.get(&key).is_none_or(|(_, known)| {
                is_newer(VersionPair {
                    candidate: &version,
                    known,
                })
            });
            if replace {
                fresh.insert(key, (id, version));
            }
        }
        if fresh.is_empty() {
            break;
        }
        let current: Vec<(String, String)> = fresh.into_values().collect();
        let fetched = each_in_parallel(ParallelWork {
            items: &current,
            work: |(id, version): &(String, String)| {
                feeds.iter().enumerate().find_map(|(position, feed)| {
                    let address = PackageAddress { feed, id, version };
                    match registry.get_text(&address.file("nuspec")) {
                        Ok(Some(text)) => Some(Ok((position, text))),
                        Ok(None) => None,
                        Err(error) => Some(Err(error)),
                    }
                })
            },
        });
        level = Vec::new();
        for ((id, version), found) in current.into_iter().zip(fetched) {
            let feed = match found.transpose()? {
                Some((feed, text)) => {
                    level.extend(
                        dependencies_for(NuspecQuery {
                            text: &text,
                            framework,
                        })
                        .into_iter()
                        .filter_map(|(dependency, range)| {
                            Some((dependency, lowest_version(&range)?))
                        }),
                    );
                    Some(feed)
                }
                None => {
                    skipped.push(format!(
                        "`{id} {version}` is not in any feed that Vigilante can read."
                    ));
                    None
                }
            };
            chosen.insert(id.to_lowercase(), Resolved { id, version, feed });
        }
    }
    let mut packages: Vec<Resolved> = chosen
        .into_values()
        .filter(|resolved| resolved.feed.is_some())
        .collect();
    packages.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(Resolution { packages, skipped })
}
