use std::io::Write;

use anyhow::Result;
use geo::Rect;

use super::{Entity, Error};

pub trait Render {
    fn render(&self, area: &Rect) -> Result<Vec<Entity>, Error>;
}

pub trait Draw {
    fn draw(&mut self, ent: Entity);
    fn dump<W: Write>(&self, out: &mut W) -> Result<()>;
    fn draw_all<V: IntoIterator<Item = Entity>>(&mut self, entities: V) {
        for ent in entities {
            self.draw(ent);
        }
    }
}
