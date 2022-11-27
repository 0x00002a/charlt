use std::io::{BufReader, Read};

use anyhow::Result;
use rlua::{Lua, StdLib};

use crate::chart::Charts;

pub fn load_chart<F: Read>(f: &mut F) -> Result<Charts> {
    let mut buf = Vec::new();
    f.read_to_end(&mut buf);
    let lua = Lua::new();
    lua.load_from_std_lib(StdLib::all())?;
    lua.context(|c| {
        let chunk = c.load(&buf);
        Ok(chunk.eval()?)
    })
}
