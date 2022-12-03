use std::io::{BufReader, BufWriter};

use anyhow::{anyhow, Result};
use api::InputFormat;
use clap::{builder::PossibleValue, Parser, ValueEnum};
use kurbo::{Rect, Size};
use piet::RenderContext;
use render::Render;
use serde::{Deserialize, Serialize};

mod api;
mod chart;
mod render;
mod serde_lua;
mod utils;

use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum OutputFormat {
    #[serde(alias = "pdf")]
    Pdf,
    #[serde(alias = "svg")]
    Svg,
    #[serde(alias = "png")]
    Png,
}

impl OutputFormat {
    fn extension(&self) -> &Path {
        match &self {
            OutputFormat::Pdf => "pdf".as_ref(),
            OutputFormat::Svg => "svg".as_ref(),
            OutputFormat::Png => "png".as_ref(),
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
fn do_render<R: RenderContext>(args: &CliArgs, r: &mut R) -> Result<()> {
    let mut input = BufReader::new(std::fs::File::open(args.input.to_owned())?);
    let chart = api::load_chart(
        &mut input,
        args.input_format
            .or_else(|| InputFormat::from_path(args.input.as_ref()))
            .ok_or(anyhow!("unknown input format"))?,
    )?;
    let size = Size::new(args.width as f64, args.height as f64);
    chart.render(&Rect::from_points((0.0, 0.0), (size.width, size.height)), r)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    let size = Size::new(args.width as f64, args.height as f64);
    let render = |surface: &dyn AsRef<cairo::Surface>| {
        do_render(
            &args,
            &mut piet_cairo::CairoRenderContext::new(&piet_cairo::cairo::Context::new(surface)?),
        )
    };
    match args.output_format.unwrap_or_else(|| {
        let p: &Path = args.output.as_ref();
        p.try_into().expect("unknown output format")
    }) {
        OutputFormat::Pdf => render(&cairo::PdfSurface::new(
            size.width,
            size.height,
            args.output.clone(),
        )?),
        OutputFormat::Svg => render(&cairo::SvgSurface::new(
            size.width,
            size.height,
            args.output.clone().into(),
        )?),
        OutputFormat::Png => {
            let surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32,
                size.width as i32,
                size.height as i32,
            )?;
            render(&surface)?;
            surface.write_to_png(&mut BufWriter::new(std::fs::File::create(&args.output)?))?;
            Ok(())
        }
    }
}
