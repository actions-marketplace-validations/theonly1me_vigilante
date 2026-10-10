use super::pom::{Coordinates, Dependency, Pom};
use super::properties::{Properties, PropertiesInput};
use crate::Error;

const MAXIMUM_PARENTS: usize = 16;
const MAXIMUM_IMPORT_DEPTH: usize = 6;
const IMPORT_SCOPE: &str = "import";
const DEFAULT_PACKAGING: &str = "jar";

pub trait PomSource {
    fn pom(&mut self, coordinates: &Coordinates) -> Result<Option<Pom>, Error>;
}

pub struct Effective {
    pub coordinates: Coordinates,
    pub packaging: String,
    pub management: Vec<Dependency>,
    pub dependencies: Vec<Dependency>,
}

pub struct EffectiveRequest<'a> {
    pub pom: Pom,
    pub source: &'a mut dyn PomSource,
    pub depth: usize,
}

#[derive(Default)]
struct Ordered {
    items: Vec<Dependency>,
}

impl Ordered {
    fn upsert(&mut self, item: Dependency) {
        match self
            .items
            .iter_mut()
            .find(|known| known.key() == item.key())
        {
            Some(known) => *known = item,
            None => self.items.push(item),
        }
    }

    fn insert_missing(&mut self, item: Dependency) {
        if !self.items.iter().any(|known| known.key() == item.key()) {
            self.items.push(item);
        }
    }
}

pub fn effective(request: EffectiveRequest) -> Result<Effective, Error> {
    let EffectiveRequest { pom, source, depth } = request;
    let mut chain = vec![pom];
    while chain.len() < MAXIMUM_PARENTS {
        let Some(parent) = chain.last().and_then(|pom| pom.parent.clone()) else {
            break;
        };
        match source.pom(&parent.coordinates)? {
            Some(found) => chain.push(found),
            None => break,
        }
    }
    let child = &chain[0];
    let parent = child.parent.as_ref().map(|parent| &parent.coordinates);
    let group = child
        .group
        .clone()
        .or_else(|| parent.map(|parent| parent.group.clone()))
        .unwrap_or_default();
    let version = child
        .version
        .clone()
        .or_else(|| parent.map(|parent| parent.version.clone()))
        .unwrap_or_default();
    let properties = Properties::new(PropertiesInput {
        chain: &chain,
        group: &group,
        version: &version,
    });
    let mut management = Ordered::default();
    let mut dependencies = Ordered::default();
    for pom in chain.iter().rev() {
        for item in &pom.management {
            management.upsert(properties.dependency(item));
        }
        for item in &pom.dependencies {
            dependencies.upsert(properties.dependency(item));
        }
    }
    let imports: Vec<Dependency> = management
        .items
        .iter()
        .filter(|item| item.scope.as_deref() == Some(IMPORT_SCOPE))
        .cloned()
        .collect();
    management
        .items
        .retain(|item| item.scope.as_deref() != Some(IMPORT_SCOPE));
    if depth < MAXIMUM_IMPORT_DEPTH {
        for import in imports {
            let coordinates = Coordinates {
                group: import.group.clone(),
                artifact: import.artifact.clone(),
                version: import.version.clone().unwrap_or_default(),
            };
            let Some(bom) = source.pom(&coordinates)? else {
                continue;
            };
            let imported = effective(EffectiveRequest {
                pom: bom,
                source,
                depth: depth + 1,
            })?;
            for item in imported.management {
                management.insert_missing(item);
            }
        }
    }
    let dependencies = dependencies
        .items
        .into_iter()
        .map(|mut item| {
            if let Some(managed) = management
                .items
                .iter()
                .find(|managed| managed.key() == item.key())
            {
                item.version = item.version.or_else(|| managed.version.clone());
                item.scope = item.scope.or_else(|| managed.scope.clone());
                if item.exclusions.is_empty() {
                    item.exclusions = managed.exclusions.clone();
                }
            }
            item
        })
        .collect();
    Ok(Effective {
        coordinates: Coordinates {
            group: properties.interpolate(&group),
            artifact: child.artifact.clone(),
            version: properties.interpolate(&version),
        },
        packaging: child
            .packaging
            .clone()
            .unwrap_or_else(|| DEFAULT_PACKAGING.to_string()),
        management: management.items,
        dependencies,
    })
}
