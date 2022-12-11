use std::fs::File;

use anyhow::{anyhow, Result};
use api::InputFormat;
use clap::{builder::PossibleValue, Parser, ValueEnum};
use plotters::{
    coord::Shift,
    prelude::{BitMapBackend, ChartBuilder, DrawingArea, DrawingBackend, IntoDrawingArea},
};
use render::Render;
use serde::{Deserialize, Serialize};

mod api;
mod chart;
mod palette;
mod render;
mod serde_lua;
mod utils;

use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum OutputFormat {
    #[serde(alias = "svg")]
    Svg,
    #[serde(alias = "png")]
    Png,
    #[serde(alias = "pdf")]
    Pdf,
}

impl OutputFormat {
    fn extension(&self) -> &Path {
        match &self {
            OutputFormat::Svg => "svg".as_ref(),
            OutputFormat::Png => "png".as_ref(),
            OutputFormat::Pdf => "pdf".as_ref(),
        }
    }
}

impl TryFrom<&Path> for OutputFormat {
    type Error = ();

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let exten = value.extension().ok_or(())?;
        for fmt in Self::value_variants() {
            if fmt.extension() == exten {
                return Ok(fmt.clone());
            }
        }
        Err(())
    }
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(name = "INPUT")]
    input: std::path::PathBuf,

    #[arg(name = "OUTPUT", short = 'o')]
    output: std::path::PathBuf,

    #[arg(long, default_value_t = 600, help = "width of chart")]
    width: u32,

    #[arg(long, default_value_t = 400, help = "height of chart")]
    height: u32,

    #[arg(
        long,
        alias = "to",
        help = "output format to use, if not provided deduced from extension"
    )]
    output_format: Option<OutputFormat>,

    #[arg(
        long,
        alias = "from",
        help = "input format to use, if not provided deduced from extension"
    )]
    input_format: Option<InputFormat>,
}
impl ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Pdf, Self::Svg, Self::Png]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match &self {
            OutputFormat::Pdf => PossibleValue::new("pdf"),
            OutputFormat::Svg => PossibleValue::new("svg"),
            OutputFormat::Png => PossibleValue::new("png"),
        })
    }
}
fn do_render<DB: DrawingBackend>(args: &CliArgs, root: DrawingArea<DB, Shift>) -> Result<()> {
    let mut builder = ChartBuilder::on(&root);
    let chart = api::load_chart(
        &mut File::open(&args.input)?,
        args.input_format
            .or_else(|| InputFormat::from_path(args.input.as_ref()))
            .ok_or(anyhow!("unknown input format"))?,
    )?;
    root.fill(&plotters::style::WHITE)
        .map_err(|e| anyhow!(e.to_string()))?;
    chart.render(&mut builder)?;
    root.present().map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    let size = (args.width, args.height);
    match args
        .output_format
        .or_else(|| args.output.as_path().try_into().ok())
        .expect("unknown output format")
    {
        OutputFormat::Svg => do_render(
            &args,
            plotters::backend::SVGBackend::new(&args.output, size).into_drawing_area(),
        ),
        OutputFormat::Png => do_render(
            &args,
            BitMapBackend::new(&args.output, size).into_drawing_area(),
        ),
        OutputFormat::Pdf => {
            let surface =
                cairo::PdfSurface::new(args.width as f64, args.height as f64, &args.output)?;
            do_render(
                &args,
                plotters_cairo::CairoBackend::new(&cairo::Context::new(&surface)?, size)?
                    .into_drawing_area(),
            )
        }
    }
}
