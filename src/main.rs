use std::io::BufReader;

use anyhow::Result;
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
}

impl OutputFormat {
    fn extension(&self) -> &Path {
        match &self {
            OutputFormat::Pdf => "pdf".as_ref(),
            OutputFormat::Svg => "svg".as_ref(),
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
        help = "output format to use, if not provided calculated from extension"
    )]
    format: Option<OutputFormat>,
}
impl ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Pdf, Self::Svg]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match &self {
            OutputFormat::Pdf => PossibleValue::new("pdf"),
            OutputFormat::Svg => PossibleValue::new("svg"),
        })
    }
}
fn do_render<R: RenderContext>(args: &CliArgs, r: &mut R) -> Result<()> {
    let mut input = BufReader::new(std::fs::File::open(args.input.to_owned())?);
    let chart = api::load_chart(&mut input)?;
    let size = Size::new(args.width as f64, args.height as f64);
    chart.render(&Rect::from_points((0.0, 0.0), (size.width, size.height)), r)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    let size = Size::new(args.width as f64, args.height as f64);
    match args.format.unwrap_or_else(|| {
        let p: &Path = args.output.as_ref();
        p.try_into().expect("unknown output format")
    }) {
        OutputFormat::Pdf => {
            let surface = cairo::PdfSurface::new(size.width, size.height, args.output.clone())?;
            do_render(
                &args,
                &mut piet_cairo::CairoRenderContext::new(&piet_cairo::cairo::Context::new(
                    surface,
                )?),
            )
        }
        OutputFormat::Svg => {
            let mut render = piet_svg::RenderContext::new(size);
            do_render(&args, &mut render)?;
            let buf = std::io::BufWriter::new(std::fs::File::create(args.output.clone())?);
            render.write(buf)?;
            Ok(())
            /*let surface =
                cairo::SvgSurface::new(size.width, size.height, args.output.clone().into())?;
            do_render(
                &args,
                &mut piet_cairo::CairoRenderContext::new(&cairo::Context::new(surface)?),
            )*/
        }
    }
}
