#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![no_std]

pub mod ir;

pub mod ast;

#[cfg(test)]
mod test_util;

/// Discriminator for zkcir's CLI to parse the output.
pub const START_DISCRIMINATOR: &str = "<ZKCIR_JSON_START>";

/// Discriminator for zkcir's CLI to parse the output.
pub const END_DISCRIMINATOR: &str = "<ZKCIR_JSON_END>";
