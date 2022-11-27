use std::io::{BufReader, BufWriter};

use anyhow::Result;
use clap::Parser;

mod api;
mod chart;
mod render;
mod utils;

#[derive(Parser)]
struct CliArgs {
    #[arg(name = "INPUT")]
    input: std::path::PathBuf,

    #[arg(name = "OUTPUT", short, long)]
    output: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let mut input = BufReader::new(std::fs::File::open(args.input)?);
    let chart = api::load_chart(&mut input);

    let mut output = BufWriter::new(std::fs::File::create(args.output)?);

    Ok(())
}
