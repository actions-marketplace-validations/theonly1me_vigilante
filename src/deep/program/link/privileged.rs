use super::Linker;
use crate::Power;
use crate::deep::graph::{Sink, Target};

pub struct PrivilegedUse<'a> {
    pub file: usize,
    pub specifier: &'a str,
    pub member: Option<&'a str>,
    pub sink_name: String,
}

impl Linker<'_> {
    pub(super) fn privileged(&mut self, wanted: PrivilegedUse) -> Vec<Target> {
        let PrivilegedUse {
            file,
            specifier,
            member,
            sink_name: name,
        } = wanted;
        let Some(privileges) = self.files[file].privileged.get(specifier) else {
            return Vec::new();
        };
        privileges
            .iter()
            .filter(|privilege| {
                !member.is_some_and(|member| privilege.inert.iter().any(|inert| inert == member))
            })
            .map(|privilege| Sink {
                name: name.clone(),
                power: Power::new(privilege.capability),
                rule: privilege.rule.clone(),
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|sink| Target::Sink(self.graph.add_sink(sink)))
            .collect()
    }
}
