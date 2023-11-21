pub mod ir;

mod ast;

#[cfg(test)]
mod test_util;

/// Discriminator for zkcir's CLI to parse the output.
pub const START_DISCRIMINATOR: &str = "<ZKCIR_JSON_START>";

/// Discriminator for zkcir's CLI to parse the output.
pub const END_DISCRIMINATOR: &str = "<ZKCIR_JSON_END>";
