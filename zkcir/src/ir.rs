use serde::Serialize;
use serde_json;

use crate::END_DISCRIMINATOR;
use crate::START_DISCRIMINATOR;

#[derive(Serialize)]
struct Config {
    num_wires: Option<u64>,
}

#[derive(Serialize)]
pub struct CirBuilder {
    config: Config,
}

impl CirBuilder {
    pub fn new() -> Self {
        CirBuilder {
            config: Config { num_wires: None },
        }
    }

    pub fn num_wires(mut self, num: u64) -> Self {
        self.config.num_wires = Some(num);
        self
    }

    pub fn to_string(self) -> Result<String, &'static str> {
        serde_json::to_string_pretty(&self).map_err(|_| "Failed serializing to json")
    }

    /// Appends discriminator to the start and end so zkcir's CLI can parse the output. You likely want `to_string`
    /// instead.
    pub fn to_cli_string(self) -> Result<String, &'static str> {
        Ok(format!(
            "{START_DISCRIMINATOR}{}\n{END_DISCRIMINATOR}",
            self.to_string()?
        ))
    }
}

impl Default for CirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::test_ir_string;

    use super::*;

    #[test]
    fn test_valid_cir() {
        test_ir_string(
            "valid_cir",
            CirBuilder::new().num_wires(10).to_string().unwrap(),
        );
    }

    #[test]
    fn test_no_wires() {
        test_ir_string("test_no_wires", CirBuilder::new().to_string().unwrap());
    }
}
