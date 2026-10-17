use std::ops::Range;

use super::ArgumentFacts;

#[derive(Default)]
pub struct CallFacts {
    sites: Vec<CallSite>,
}

pub struct CallSite {
    pub range: Range<usize>,
    pub arguments: Range<usize>,
    pub facts: ArgumentFacts,
}

impl CallFacts {
    pub fn push(&mut self, site: CallSite) {
        self.sites.push(site);
    }

    pub fn at(&self, position: usize) -> Option<ArgumentFacts> {
        let holding = || {
            self.sites
                .iter()
                .filter(move |site| site.range.contains(&position))
        };
        let chain = holding()
            .filter(|site| !site.arguments.contains(&position))
            .map(|site| site.facts)
            .reduce(ArgumentFacts::merged);
        chain.or_else(|| {
            holding()
                .min_by_key(|site| site.range.len())
                .map(|site| site.facts)
        })
    }
}
