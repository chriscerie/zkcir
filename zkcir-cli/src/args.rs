use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// emits ir as json; this and/or `source` must be enabled
    #[arg(long)]
    pub json: bool,

    /// emits ir as source code; this and/or `json` must be enabled
    #[arg(long)]
    pub source: bool,

    /// replaces output file(s) when they already exist
    #[arg(long)]
    pub allow_dirty: bool,

    /// name of the output file(s) without extension. if not provided, tries to infer name from entry point
    #[arg(long, short)]
    pub name: Option<String>,

    /// if starts with `cargo`, executes the command. otherwise forwards args to `cargo run`
    #[arg(last = true)]
    pub cargo_args: Vec<String>,
}
