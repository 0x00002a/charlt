use std::io::Write;

use anyhow::Result;

use super::Entity;

pub trait Render {
    fn render(&self) -> Vec<Entity>;
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
