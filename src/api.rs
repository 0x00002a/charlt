use std::io::Read;

use anyhow::Result;
use clap::{builder::PossibleValue, ValueEnum};
use rlua::{Lua, StdLib};

use crate::{chart::Charts, serde_lua::from_lua};

#[derive(Clone, Copy, Debug)]
pub enum InputFormat {
    Yaml,
    Lua,
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
        InputFormat::Yaml => Ok(serde_yaml::from_reader(f)?),
        InputFormat::Lua => {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            let lua = Lua::new();
            lua.load_from_std_lib(StdLib::ALL_NO_DEBUG)?;
            lua.context(|c| {
                let chunk = c.load(&buf);
                Ok(from_lua(chunk.eval()?)?)
            })
        }
    }
}
