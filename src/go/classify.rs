use super::capslock::CapabilityInfo;
use super::mapping::{self, EvidenceStart, Module};
use super::{GoModules, OwnedFolder};
use crate::config::Config;
use crate::deep::{OwnPackagePath, Reach, ReachKind, note_kind_for_own_package};
use crate::{Error, Scope, Sighting, Surface};

pub enum Classified {
    Sighting(Sighting),
    Reach(Reach),
}

pub struct ClassifyInput<'a> {
    pub info: &'a CapabilityInfo,
    pub module: &'a Module,
    pub modules: &'a GoModules,
    pub config: &'a Config,
}

pub fn classify(input: ClassifyInput) -> Result<Option<Classified>, Error> {
    let ClassifyInput {
        info,
        module,
        modules,
        config,
    } = input;
    let power = mapping::power_of(&info.capability_name)?;
    let owned_folder = modules
        .folder_of(&info.package_dir)
        .filter(|folder| modules.is_owned_by(OwnedFolder { folder, module }));
    let (start_package, folder, kind) = match owned_folder {
        Some(folder) => {
            let is_own_code = info.path.split_last().is_none_or(|(_, before)| {
                before
                    .iter()
                    .all(|function| modules.is_repo_package(&function.package))
            });
            let kind = note_kind_for_own_package(OwnPackagePath {
                capability: power.capability,
                stays_in_repository: is_own_code,
            });
            (info.package_dir.as_str(), folder, kind)
        }
        None => {
            let Some(root) = info.path.first() else {
                return Ok(None);
            };
            let Some(folder) = modules.folder_of(&root.package) else {
                return Ok(None);
            };
            (
                root.package.as_str(),
                folder,
                Some(ReachKind::NewPackageOnPath),
            )
        }
    };
    let evidence = mapping::evidence_of(EvidenceStart {
        info,
        package: start_package,
        folder: &folder,
    });
    if config.is_outside_analysis(&evidence.path) {
        return Ok(None);
    }
    let scope = Scope {
        surface: Surface::Code,
        path: config
            .named_scope_of(&evidence.path)
            .map_or(folder, str::to_string),
    };
    let Some(kind) = kind else {
        return Ok(Some(Classified::Sighting(Sighting {
            scope,
            power,
            evidence,
        })));
    };
    let through = info
        .path
        .iter()
        .map(|function| function.package.as_str())
        .find(|package| !modules.is_repo_package(package))
        .unwrap_or(&info.package_dir)
        .to_string();
    Ok(Some(Classified::Reach(Reach {
        package: info.package_dir.clone(),
        through,
        kind,
        power,
        scope,
        evidence,
    })))
}
