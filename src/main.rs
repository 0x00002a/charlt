use std::io::{BufReader, BufWriter};

use anyhow::Result;
use clap::Parser;
use render::{Draw, Render};

mod api;
mod chart;
mod render;
mod utils;

#[derive(Parser)]
struct CliArgs {
    #[arg(name = "INPUT")]
    input: std::path::PathBuf,

    #[arg(name = "OUTPUT", short = 'o')]
    output: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let mut input = BufReader::new(std::fs::File::open(args.input)?);
    let chart = api::load_chart(&mut input)?;

    let mut output = BufWriter::new(std::fs::File::create(args.output)?);
    let mut out_svg = render::svg::Svg::new();
    out_svg.draw_all(chart.render());
    out_svg.dump(&mut output)?;

    Ok(())
}
