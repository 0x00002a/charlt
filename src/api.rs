use std::io::{BufReader, Read};

use anyhow::Result;
use rlua::{Lua, StdLib};

use crate::{chart::Charts, serde_lua::from_lua};

pub fn load_chart<F: Read>(f: &mut F) -> Result<Charts> {
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let lua = Lua::new();
    lua.load_from_std_lib(StdLib::ALL_NO_DEBUG)?;
    lua.context(|c| {
        let chunk = c.load(&buf);
        Ok(from_lua(chunk.eval()?)?)
    })
}
