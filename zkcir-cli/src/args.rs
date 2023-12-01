use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// name of the target example circuit; must be in `/examples`
    #[arg(short, long)]
    pub example: Option<String>,

    /// when enabled, emits ir as json; this and/or `source` must be enabled
    #[arg(long)]
    pub json: bool,

    /// when enabled, emits ir as source code; this and/or `json` must be enabled
    #[arg(long)]
    pub source: bool,

    /// when enabled, replaces output file(s) when they already exist
    #[arg(long)]
    pub allow_dirty: bool,
}

pub fn get_args() -> Args {
    Args::parse()
}
