use super::{ScanRequest, Scanner, ScannerInput, Selection};
use crate::config::Config;
use crate::deep::Skip;
use crate::{Error, PowerMap};

pub struct ScanReport {
    pub map: PowerMap,
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
    Ok(ScanReport {
        map: scanned.map,
        not_analyzed: scanned.skipped,
    })
}
