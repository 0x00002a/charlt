use std::io::Write;

use anyhow::Result;
use kurbo::Rect;
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};

use super::{Entity, Error, TextInfo};

pub trait Render {
    type Error: std::error::Error;
    fn render<P: RenderContext>(&self, area: &Rect, r: &mut P) -> Result<(), Self::Error>;
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

pub trait RenderContextExt {
    fn render_text(&mut self, pt: kurbo::Point, info: &TextInfo);
}
impl<R: RenderContext> RenderContextExt for R {
    fn render_text(&mut self, pt: kurbo::Point, info: &TextInfo) {
        let t = self.text().new_text_layout(info.content);
        if let Some(f) = info.font {
            t = t.font(f.family, f.size);
        }
        if let Some(c) = info.colour {
            t = t.text_color(c);
        }
        self.draw_text(&t.build().unwrap(), pt);
    }
}
