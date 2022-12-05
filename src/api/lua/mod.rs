const FILES: [&str; 1] = [include_str!("./csv.lua")];

use anyhow::{anyhow, Result};
use rlua::{self};

fn access_global_ns<'lua>(
    ctx: rlua::Context<'lua>,
    ns: &Vec<String>,
) -> Result<(rlua::Table<'lua>, String)> {
    let mut last_head = ctx.globals();
    let mut head = rlua::Value::Nil;
    for n in ns.iter().map(|n| AsRef::<str>::as_ref(n)) {
        match head {
            rlua::Value::Table(t) => {
                last_head = t.clone();
                head = t.get::<_, rlua::Value<'lua>>(n)?;
            }
            rlua::Value::Nil => {
                let tbl = ctx.create_table()?;
                last_head.set(n, tbl.clone())?;
                head = rlua::Value::Table(tbl);
            }
            _ => return Err(anyhow!("namespace selector went through non-table value?")),
        }
    }
    Ok((
        last_head,
        ns.last().ok_or(anyhow!("namespace empty?"))?.to_owned(),
    ))
}

pub fn load_api(ctx: rlua::Context) -> Result<()> {
    for f in FILES {
        let module: rlua::Table = ctx.load(f).eval()?;
        let namespace: Vec<String> = module
            .get::<_, String>("namespace")
            .map_err(|e| anyhow!("failed to get namespace: {}", e))?
            .split(',')
            .map(|s| s.to_owned())
            .collect();
        let mod_val: rlua::Value = module
            .get("module")
            .map_err(|e| anyhow!("failed to get module: {}", e))?;
        let (ns, last_ns) = access_global_ns(ctx, &namespace)?;
        ns.set(last_ns, mod_val)?;
    }
    Ok(())
}
