use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::ir::CirBuilder;

lazy_static::lazy_static! {
    static ref TEST_PROJECTS_ROOT: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("cir_test_snapshots");
}

pub fn test_ir_string(test_name: &str, cir: &CirBuilder) {
    let test_ir_path = TEST_PROJECTS_ROOT.join(test_name).with_extension("ir.txt");
    let test_json_path = TEST_PROJECTS_ROOT.join(test_name).with_extension("json");

    let ir = format!("{cir:#?}");
    let cir_json = cir.to_string().unwrap();

    if let Ok(expected) = fs::read_to_string(&test_json_path) {
        pretty_assertions::assert_str_eq!(
            // Must normalize newline characters otherwise testing on windows locally passes but fails
            // in github actions environment
            &expected.replace("\r\n", "\n"),
            &cir_json.replace("\r\n", "\n")
        );
    } else {
        let mut output_file =
            fs::File::create(test_json_path).expect("couldn't create output file");
        output_file
            .write_all(cir_json.as_bytes())
            .expect("couldn't write to output file.");
    }

    if let Ok(expected) = fs::read_to_string(&test_ir_path) {
        pretty_assertions::assert_str_eq!(
            // Must normalize newline characters otherwise testing on windows locally passes but fails
            // in github actions environment
            &expected.replace("\r\n", "\n"),
            &ir.replace("\r\n", "\n")
        );
    } else {
        let mut output_file = fs::File::create(test_ir_path).expect("couldn't create output file");
        output_file
            .write_all(ir.as_bytes())
            .expect("couldn't write to output file.");
    }
}
