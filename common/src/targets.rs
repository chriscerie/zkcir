use core::fmt;
use core::hash::Hash;
use std::collections::HashSet;
use std::hash::Hasher;

#[derive(Debug, serde::Serialize)]
pub enum TargetFramework {
    Plonky2,
    Halo2,
}

impl fmt::Display for TargetFramework {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TargetFramework::Plonky2 => write!(f, "plonky2"),
            TargetFramework::Halo2 => write!(f, "halo2"),
        }
    }
}

impl TargetFramework {
    #[must_use]
    pub fn get_dependencies(&self) -> HashSet<PatchedDependency> {
        match self {
            TargetFramework::Plonky2 => [PatchedDependency {
                git_url: "https://github.com/chriscerie/plonky2.git".to_string(),
                dependency_names: [
                    "plonky2".to_string(),
                    "plonky2_evm".to_string(),
                    "plonky2_field".to_string(),
                    "plonky2_maybe_rayon".to_string(),
                    "starky".to_string(),
                    "plonky2_util".to_string(),
                ]
                .into(),
            }]
            .into_iter()
            .collect(),
            TargetFramework::Halo2 => [PatchedDependency {
                git_url: "https://github.com/csirlin/halo2".to_string(),
                dependency_names: [
                    "halo2".to_string(),
                    "halo2_proofs".to_string(),
                    "halo2_gadgets".to_string(),
                ]
                .into(),
            }]
            .into_iter()
            .collect(),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct PatchedDependency {
    pub git_url: String,
    pub dependency_names: HashSet<String>,
}

impl Hash for PatchedDependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.git_url.hash(state);
    }
}
