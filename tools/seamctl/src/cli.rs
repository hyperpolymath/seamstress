use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "seamctl", version, about = "Seamstress core CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Validate seam records against schema + policy checks
    Validate {
        /// Root directory to scan (default: current directory)
        #[arg(long)]
        root: Option<String>,
        /// Path to seam record schema (default: seams/schema/seam-record.schema.json)
        #[arg(long)]
        schema: Option<String>,
    },
    /// Build a component/seam dependency graph
    Graph {
        #[arg(long, default_value = "json")]
        format: String,
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        root: Option<String>,
    },
    /// Generate an AsciiDoc report
    Report {
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        root: Option<String>,
    },
}

impl Cli {
    pub fn exec(self) -> Result<()> {
        match self.cmd {
            Command::Validate { root, schema } => {
                crate::validate::run(root.as_deref(), schema.as_deref())
            }
            Command::Graph { format, out, root } => {
                crate::graph::run(root.as_deref(), &format, out.as_deref())
            }
            Command::Report { out, root } => crate::report::run(root.as_deref(), out.as_deref()),
        }
        .with_context(|| "command failed")
    }
}
