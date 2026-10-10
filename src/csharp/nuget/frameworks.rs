#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Family {
    Classic,
    Standard,
    CoreApp,
    Modern,
}

struct Framework {
    family: Family,
    version: Vec<u32>,
    platform: Option<String>,
}

pub struct FrameworkChoice<'a> {
    pub project: &'a str,
    pub candidates: &'a [&'a str],
}

pub fn best_framework(choice: FrameworkChoice) -> Option<String> {
    let FrameworkChoice {
        project,
        candidates,
    } = choice;
    let project = parse(project)?;
    candidates
        .iter()
        .filter_map(|candidate| {
            let framework = parse(candidate)?;
            let rank = rank(Ranked {
                project: &project,
                candidate: &framework,
            })?;
            Some((
                (rank, framework.platform.is_some(), framework.version),
                candidate.to_string(),
            ))
        })
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, candidate)| candidate)
}

struct Ranked<'a> {
    project: &'a Framework,
    candidate: &'a Framework,
}

fn rank(ranked: Ranked) -> Option<u8> {
    let Ranked { project, candidate } = ranked;
    if let Some(platform) = &candidate.platform
        && project.platform.as_deref() != Some(platform.as_str())
    {
        return None;
    }
    let is_older = candidate.version <= project.version;
    let standard_limit = match project.family {
        Family::Classic if project.version >= vec![4, 6, 1] => vec![2, 0],
        Family::Classic => return (candidate.family == Family::Classic && is_older).then_some(3),
        Family::Standard => project.version.clone(),
        Family::CoreApp | Family::Modern => vec![2, 1],
    };
    match (project.family, candidate.family) {
        (Family::Modern, Family::Modern) | (Family::CoreApp, Family::CoreApp) if is_older => {
            Some(4)
        }
        (Family::Modern, Family::CoreApp) => Some(3),
        (Family::Classic, Family::Classic) if is_older => Some(3),
        (_, Family::Standard) if candidate.version <= standard_limit => Some(2),
        (Family::Modern, Family::Classic) => Some(1),
        _ => None,
    }
}

fn parse(text: &str) -> Option<Framework> {
    let lowered = text.trim().to_lowercase();
    let (name, platform) = match lowered.split_once('-') {
        Some((name, platform)) => (
            name.to_string(),
            Some(
                platform
                    .trim_end_matches(|character: char| {
                        character.is_ascii_digit() || character == '.'
                    })
                    .to_string(),
            ),
        ),
        None => (lowered.clone(), None),
    };
    let dotted = |rest: &str| -> Option<Vec<u32>> {
        rest.split('.').map(|part| part.parse().ok()).collect()
    };
    let (family, version) = if let Some(rest) = name
        .strip_prefix(".netstandard")
        .or_else(|| name.strip_prefix("netstandard"))
    {
        (Family::Standard, dotted(rest)?)
    } else if let Some(rest) = name
        .strip_prefix(".netcoreapp")
        .or_else(|| name.strip_prefix("netcoreapp"))
    {
        (Family::CoreApp, dotted(rest)?)
    } else if let Some(rest) = name.strip_prefix(".netframework") {
        (Family::Classic, dotted(rest)?)
    } else if let Some(rest) = name.strip_prefix("net") {
        if rest.contains('.') {
            (Family::Modern, dotted(rest)?)
        } else {
            (
                Family::Classic,
                rest.chars()
                    .map(|digit| digit.to_digit(10))
                    .collect::<Option<Vec<u32>>>()?,
            )
        }
    } else {
        return None;
    };
    Some(Framework {
        family,
        version,
        platform,
    })
}
