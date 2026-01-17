mod cli;
mod graph;
mod model;
mod report;
mod validate;

use anyhow::Result;
use clap::Parser;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cmd = cli::Cli::parse();
    cmd.exec()
}
