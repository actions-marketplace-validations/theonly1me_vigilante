use crate::code::Rules;
use crate::{Evidence, Scope, Sighting, SourceFile, Surface};

pub struct TestFile<'a> {
    pub rules: &'a Rules,
    pub scope: String,
    pub source: SourceFile<'a>,
}

pub fn test_sightings(file: TestFile) -> Vec<Sighting> {
    file.rules
        .detect(file.source)
        .into_iter()
        .filter(|detection| detection.power.capability.surface() == Surface::Code)
        .map(|detection| Sighting {
            scope: Scope {
                surface: Surface::Test,
                path: file.scope.clone(),
            },
            power: detection.power,
            evidence: Evidence {
                path: file.source.path.to_string(),
                line: detection.line,
                rule: detection.rule,
                call_path: Vec::new(),
            },
        })
        .collect()
}
