use super::Entity;

pub trait Render {
    fn render(&self) -> Vec<Entity>;
}

pub trait Draw {
    fn draw(self, ent: Entity) -> Self;
}
