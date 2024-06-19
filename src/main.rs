use std::process::Command;

use clap::Parser as _;
use cli::Cli;
use search::Search;

mod cli;
mod resolv;
mod search;

fn main() -> anyhow::Result<()> {
    let cli_args = Cli::parse();

    let args = ["/all"];

    let out = Command::new(cli_args.command)
        .args(args)
        .output()
        .expect("Error running ipconfig.exe command.");

    let all_output = String::from_utf8(out.stdout)
        .expect("Could not create string stdout. Perhaps it was not UTF-8 encoded?");

    Search::from(all_output.as_str()).generate_resolv_conf(Box::new(std::io::stdout()))?;

    Ok(())
}
