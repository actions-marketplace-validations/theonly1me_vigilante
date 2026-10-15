use std::cmp::Ordering;

const MINIMUM_PARTS: usize = 3;

pub fn normalized(text: &str) -> String {
    let without_metadata = text.split('+').next().unwrap_or(text).trim().to_lowercase();
    let (release, prerelease) = match without_metadata.split_once('-') {
        Some((release, prerelease)) => (release.to_string(), Some(prerelease.to_string())),
        None => (without_metadata.clone(), None),
    };
    let mut parts: Vec<String> = release
        .split('.')
        .map(|part| {
            part.parse::<u64>()
                .map_or_else(|_| part.to_string(), |number| number.to_string())
        })
        .collect();
    while parts.len() < MINIMUM_PARTS {
        parts.push("0".to_string());
    }
    if parts.len() > MINIMUM_PARTS && parts[MINIMUM_PARTS..].iter().all(|part| part == "0") {
        parts.truncate(MINIMUM_PARTS);
    }
    let mut version = parts.join(".");
    if let Some(prerelease) = prerelease {
        version.push('-');
        version.push_str(&prerelease);
    }
    version
}

pub fn lowest_version(range: &str) -> Option<String> {
    let range = range.trim();
    if range.is_empty() || range.contains(['$', '*']) {
        return None;
    }
    let minimum = if range.starts_with(['[', '(']) {
        let inner = range
            .trim_start_matches(['[', '('])
            .trim_end_matches([']', ')']);
        inner.split(',').next().unwrap_or_default().trim()
    } else {
        range
    };
    (!minimum.is_empty()).then(|| normalized(minimum))
}

pub struct VersionPair<'a> {
    pub candidate: &'a str,
    pub known: &'a str,
}

pub fn is_newer(pair: VersionPair) -> bool {
    let parse = |text: &str| -> (Vec<u64>, Option<String>) {
        let (release, prerelease) = match text.split_once('-') {
            Some((release, prerelease)) => (release, Some(prerelease.to_string())),
            None => (text, None),
        };
        (
            release
                .split('.')
                .map(|part| part.parse().unwrap_or(0))
                .collect(),
            prerelease,
        )
    };
    let (candidate_release, candidate_prerelease) = parse(pair.candidate);
    let (known_release, known_prerelease) = parse(pair.known);
    let order = candidate_release.cmp(&known_release).then_with(|| {
        match (candidate_prerelease, known_prerelease) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(left), Some(right)) => left.cmp(&right),
        }
    });
    order == Ordering::Greater
}
