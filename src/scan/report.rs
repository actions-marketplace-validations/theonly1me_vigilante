use super::{ScanRequest, Scanner, ScannerInput, Selection};
use crate::config::Config;
use crate::deep::{Risk, Skip};
use crate::{Error, PowerMap};

pub struct ScanReport {
    pub map: PowerMap,
    pub examples: PowerMap,
    pub risks: Vec<Risk>,
    pub not_analyzed: Vec<Skip>,
}

pub fn scan_report(request: ScanRequest) -> Result<ScanReport, Error> {
    let located = request.source.locate(&request.repository)?;
    let config = Config::load(located.tree.as_ref())?;
    let scanned = Scanner::new(ScannerInput {
        tree: located.tree.as_ref(),
        rules_tree: located.tree.as_ref(),
        config: &config,
        location: located.location(),
    })?
    .scan(&Selection::Everything)?;
    let mut examples = PowerMap::default();
    for sighting in scanned.examples {
        examples.insert(sighting);
    }
    Ok(ScanReport {
        map: scanned.map,
        examples,
        risks: scanned.risks,
        not_analyzed: scanned.skipped,
    })
}
