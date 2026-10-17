use crate::deep::Risk;
use crate::scope::Scopes;
use crate::surfaces::{self, Placement, SurfaceRisk};
use crate::{Capability, Detection, Evidence, Scope, Sighting, Surface};

const SURFACE_RISK_RULE: &str = "github-workflow";

pub struct Placing<'a> {
    pub scopes: &'a Scopes<'a>,
    pub path: &'a str,
}

impl Placing<'_> {
    pub fn sighting_of(&self, detection: Detection) -> Sighting {
        let surface = detection.power.capability.surface();
        let path = match surface {
            Surface::Code => self.scopes.code_scope_of(self.path),
            _ => self.surface_path(&detection.power.capability),
        };
        Sighting {
            scope: Scope { surface, path },
            power: detection.power,
            evidence: Evidence {
                path: self.path.to_string(),
                line: detection.line,
                rule: detection.rule,
                call_path: Vec::new(),
            },
        }
    }

    pub fn risk_of(&self, risk: SurfaceRisk) -> Risk {
        Risk {
            scope: Scope {
                surface: Surface::Ci,
                path: self.surface_path(&risk.power.capability),
            },
            identity: risk.identity,
            power: risk.power,
            class: risk.class,
            evidence: Evidence {
                path: self.path.to_string(),
                line: risk.line,
                rule: SURFACE_RISK_RULE.to_string(),
                call_path: risk.call_path,
            },
        }
    }

    fn surface_path(&self, capability: &Capability) -> String {
        surfaces::scope_path_of(Placement {
            path: self.path,
            capability: *capability,
        })
        .to_string()
    }
}
