use std::io::{BufReader, BufWriter};

use anyhow::Result;
use clap::Parser;
use geo::Rect;
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

    #[arg(long, default_value_t = 600, help = "width of chart")]
    width: u32,

    #[arg(long, default_value_t = 400, help = "height of chart")]
    height: u32,
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let mut input = BufReader::new(std::fs::File::open(args.input)?);
    let chart = api::load_chart(&mut input)?;

    let mut output = BufWriter::new(std::fs::File::create(args.output)?);
    let mut out_svg = render::svg::Svg::new();
    out_svg.draw_all(chart.render(&Rect::new(
        (0.0, 0.0),
        (args.width as f64, args.height as f64),
    ))?);
    out_svg.dump(&mut output)?;

    Ok(())
}
