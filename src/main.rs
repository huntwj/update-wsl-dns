use std::process::Command;

use search::Search;

mod resolv;
mod search;

fn main() -> anyhow::Result<()> {
    let command = "/mnt/c/Windows/System32/ipconfig.exe";
    let args = ["/all"];

    let out = Command::new(command)
        .args(args)
        .output()
        .expect("Error running ipconfig.exe command.");

    let all_output = String::from_utf8(out.stdout)
        .expect("Could not create string stdout. Perhaps it was not UTF-8 encoded?");

    let search = Search::from(all_output.as_str());

    search.generate_resolv_conf(Box::new(std::io::stdout()))?;

    Ok(())
}
