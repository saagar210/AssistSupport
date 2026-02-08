use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = memory_kernel_outcome_cli::Cli::parse();
    memory_kernel_outcome_cli::run_cli(cli)
}
