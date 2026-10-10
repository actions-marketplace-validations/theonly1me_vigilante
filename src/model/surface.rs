use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Code,
    Ci,
    Agent,
    Build,
}

impl Surface {
    pub fn name(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Ci => "ci",
            Self::Agent => "agent",
            Self::Build => "build",
        }
    }
}
