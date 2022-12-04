use std::io::Read;

use anyhow::{anyhow, Result};
use clap::{builder::PossibleValue, ValueEnum};
use rlua::{Lua, StdLib, Table};
use serde::{de::Deserializer, Deserialize};

mod lua;

use crate::{
    chart::Charts,
    serde_lua::{self, from_lua},
};

#[derive(Clone, Copy, Debug)]
pub enum InputFormat {
    Yaml,
    Lua,
}

#[derive(Clone, Debug, Deserialize)]
struct CsvOptions {
    delim: Option<char>,
    double_quote: Option<bool>,
    flexible: Option<bool>,
    #[serde(deserialize_with = "deserialize_terminator")]
    terminator: Option<csv::Terminator>,
    quote: Option<char>,
    escape: Option<char>,
    quoting: Option<bool>,
    comment: Option<char>,
}

fn deserialize_terminator<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<csv::Terminator>, D::Error> {
    let s = Option::<u8>::deserialize(deserializer)?;
    Ok(s.map(|s| csv::Terminator::Any(s)))
}
fn load_csv_bindings<'lua>(ctx: rlua::Context<'lua>) -> Result<()> {
    let tbl = ctx.create_table()?;
    tbl.set(
        "parse_string",
        ctx.create_function(
            |_,
             (content, opts_tbl): (String, Option<Table>)|
             -> Result<(Vec<String>, Vec<Vec<String>>), rlua::Error> {
                let opts: Option<CsvOptions> = opts_tbl
                    .map(|o| serde_lua::from_lua(rlua::Value::Table(o)))
                    .transpose()
                    .map_err(|e| rlua::Error::RuntimeError(e.to_string()))?;
                let mut builder = csv::ReaderBuilder::new();
                if let Some(opts) = opts {
                    builder
                        .quoting(opts.quoting.unwrap_or(true))
                        .delimiter(opts.delim.unwrap_or(',') as u8)
                        .quote(opts.quote.unwrap_or('"') as u8)
                        .comment(opts.comment.map(|c| c as u8))
                        .double_quote(opts.double_quote.unwrap_or(true))
                        .flexible(opts.flexible.unwrap_or(false))
                        .escape(opts.escape.map(|e| e as u8))
                        .terminator(opts.terminator.unwrap_or(csv::Terminator::CRLF));
                }
                let mut p = builder.from_reader(content.as_bytes());
                Ok({
                    let records = p
                        .records()
                        .into_iter()
                        .map(|r| {
                            r.map_err(|e| rlua::Error::RuntimeError(e.to_string()))
                                .map(|r| r.into_iter().map(|s| s.to_owned()).collect())
                        })
                        .collect::<std::result::Result<Vec<_>, rlua::Error>>()?;
                    if p.has_headers() {
                        (
                            p.headers()
                                .map_err(|e| rlua::Error::RuntimeError(e.to_string()))?
                                .into_iter()
                                .map(|c| c.to_owned())
                                .collect::<Vec<_>>(),
                            records,
                        )
                    } else {
                        (vec![], records)
                    }
                })
            },
        )?,
    )?;
    ctx.globals().set("__rs", tbl)?;
    Ok(())
}

impl InputFormat {
    pub fn from_path(p: &std::path::Path) -> Option<InputFormat> {
        p.extension().and_then(|p| match p.to_str()? {
            "lua" => Some(InputFormat::Lua),
            "yaml" | "yml" => Some(InputFormat::Yaml),
            _ => None,
        })
    }
}
impl ValueEnum for InputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[InputFormat::Yaml, InputFormat::Lua]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            InputFormat::Yaml => PossibleValue::new("yaml"),
            InputFormat::Lua => PossibleValue::new("lua"),
        })
    }
}

pub fn load_chart<F: Read>(f: &mut F, fmt: InputFormat) -> Result<Charts> {
    match fmt {
        InputFormat::Yaml => Ok(serde_yaml::from_reader(f).map_err(|e| {
            anyhow!(
                "failed to deserailize input: {} at {}",
                e.to_string(),
                e.location()
                    .map(|l| format!("{}:{}", l.line(), l.column()))
                    .unwrap_or("unknown".to_owned())
            )
        })?),
        InputFormat::Lua => {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            let lua = Lua::new();
            lua.load_from_std_lib(StdLib::ALL_NO_DEBUG)?;
            lua.context(|c| {
                load_csv_bindings(c)?;
                let chunk = c.load(&buf);
                Ok(from_lua(chunk.eval()?)?)
            })
        }
    }
}
