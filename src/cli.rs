use clap::Parser;
use clap_stdin::FileOrStdin;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(default_value = "-")]
    pub input: Option<FileOrStdin>,
    #[arg(short, long)]
    pub port: Option<u16>,
    #[arg(short, long, default_value_t = false)]
    pub no_browser: bool,
}
