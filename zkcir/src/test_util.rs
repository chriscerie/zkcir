use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

lazy_static::lazy_static! {
    static ref TEST_PROJECTS_ROOT: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("cir_test_snapshots");
}

pub fn test_ir_string(test_name: &str, cir: String) {
    let test_path = TEST_PROJECTS_ROOT.join(test_name).with_extension("json");

    if let Ok(expected) = fs::read_to_string(&test_path) {
        pretty_assertions::assert_str_eq!(
            // Must normalize newline characters otherwise testing on windows locally passes but fails
            // in github actions environment
            &expected.replace("\r\n", "\n"),
            &cir.replace("\r\n", "\n")
        );
    } else {
        let mut output_file = fs::File::create(test_path).expect("couldn't create output file");
        output_file
            .write_all(cir.as_bytes())
            .expect("couldn't write to output file.");
    }
}
