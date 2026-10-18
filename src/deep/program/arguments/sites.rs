use std::ops::Range;

use super::ArgumentFacts;

#[derive(Default)]
pub struct CallFacts {
    sites: Vec<CallSite>,
    parents: Vec<Option<usize>>,
    open: Vec<usize>,
}

pub struct CallSite {
    pub range: Range<usize>,
    pub arguments: Range<usize>,
    pub facts: ArgumentFacts,
}

impl CallFacts {
    pub fn push(&mut self, site: CallSite) {
        while let Some(&last) = self.open.last() {
            if self.sites[last].range.end > site.range.start {
                break;
            }
            self.open.pop();
        }
        self.parents.push(self.open.last().copied());
        self.open.push(self.sites.len());
        self.sites.push(site);
    }

    fn holding(&self, position: usize) -> Vec<&CallSite> {
        let started = self
            .sites
            .partition_point(|site| site.range.start <= position);
        let mut current = started.checked_sub(1);
        while let Some(index) = current {
            if self.sites[index].range.contains(&position) {
                break;
            }
            current = self.parents[index];
        }
        let mut holding = Vec::new();
        while let Some(index) = current {
            holding.push(&self.sites[index]);
            current = self.parents[index];
        }
        holding.reverse();
        holding
    }

    pub fn at(&self, position: usize) -> Option<ArgumentFacts> {
        let holding = self.holding(position);
        let chain = holding
            .iter()
            .filter(|site| !site.arguments.contains(&position))
            .map(|site| site.facts)
            .reduce(ArgumentFacts::merged);
        chain.or_else(|| {
            holding
                .iter()
                .min_by_key(|site| site.range.len())
                .map(|site| site.facts)
        })
    }
}
