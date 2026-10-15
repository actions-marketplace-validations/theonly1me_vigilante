use super::{Linker, Unit};
use crate::deep::graph::{Call, Function, FunctionId, Owner, Target};

impl Linker<'_> {
    pub(super) fn fallback(&mut self, package: usize) -> FunctionId {
        if let Some(id) = self.fallbacks.get(&package) {
            return *id;
        }
        let name = self.resolver.package_name(package);
        let id = self.graph.add_function(Function {
            name: format!("{name}.*"),
            owner: Owner::Dependency {
                package: name.clone(),
            },
            path: name,
        });
        let members = self
            .in_unit
            .get(&Unit::Package(package))
            .cloned()
            .unwrap_or_default();
        for member in members {
            self.graph.add_call(Call {
                from: id,
                line: 1,
                target: Target::Function(member),
            });
        }
        self.fallbacks.insert(package, id);
        id
    }
}
