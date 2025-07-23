use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub commands: Cmds,
}

#[derive(Subcommand, Debug)]
pub enum Cmds {
    StartServer {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long)]
        port: Option<u16>,
    },

    Compile {
        #[arg(short, long)]
        file: PathBuf,
        output_file: PathBuf,
    },
}

