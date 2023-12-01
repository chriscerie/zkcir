#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![no_std]

pub mod ir;

pub mod ast;

#[cfg(test)]
mod test_util;

// Discriminators for zkcir's CLI to parse the output
pub const START_DISCRIMINATOR_JSON: &str = "<ZKCIR_JSON_START>";
pub const END_DISCRIMINATOR_JSON: &str = "<ZKCIR_JSON_END>";

pub const END_DISCRIMINATOR_SOURCE: &str = "<ZKCIR_SOURCE_END>";
pub const START_DISCRIMINATOR_SOURCE: &str = "<ZKCIR_SOURCE_START>";
