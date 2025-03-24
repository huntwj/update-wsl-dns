use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(long, default_value = "ipconfig.exe")]
    pub command: String,
}
