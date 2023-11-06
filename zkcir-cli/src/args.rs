use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// name of the target example circuit; must be in `/examples`
    #[arg(short, long)]
    pub example: Option<String>,

    /// when enabled, replaces output file(s) when they already exist
    #[arg(long)]
    pub allow_dirty: bool,
}

pub fn get_args() -> Args {
    Args::parse()
}
