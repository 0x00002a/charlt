use std::io::{BufReader, BufWriter};

use anyhow::Result;
use clap::Parser;
use kurbo::{Rect, Size, TranslateScale};
use render::Render;

mod api;
mod chart;
mod render;
mod serde_lua;
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

    let output = BufWriter::new(std::fs::File::create(args.output)?);
    let size = Size::new(args.width as f64, args.height as f64);
    let mut svg_render = piet_svg::RenderContext::new(size.clone() * 0.5);
    chart
        .render(
            &Rect::from_points((0.0, 0.0), (size.width, size.height)),
            &mut svg_render,
        )
        .expect("failed to render");
    svg_render.write(output)?;
    Ok(())
}
