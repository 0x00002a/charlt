use std::io::Write;

use super::Entity;

pub trait Render {
    fn render(&self) -> Vec<Entity>;
}

pub trait Draw {
    fn draw(self, ent: Entity) -> Self;
    fn dump<W: Write>(&self, out: &mut W);
}
