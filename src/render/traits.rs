use std::io::Write;

use anyhow::Result;
use kurbo::Rect;
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};

use super::{Error, TextInfo};

pub trait Render {
    type Error: std::error::Error;
    fn render<P: RenderContext>(&self, area: &Rect, r: &mut P) -> Result<(), Self::Error>;
}

pub trait RenderContextExt {
    fn render_text(&mut self, pt: kurbo::Point, info: &TextInfo) -> Result<(), Error>;
}

impl<R: RenderContext> RenderContextExt for R {
    fn render_text(&mut self, pt: kurbo::Point, info: &TextInfo) -> Result<(), Error> {
        let mut t = self.text().new_text_layout(info.content.to_owned());
        if let Some(f) = &info.font {
            t = t.font(f.family.clone().to_family(self)?, f.size);
        }
        if let Some(c) = info.colour {
            t = t.text_color(c);
        }
        self.draw_text(
            &(match t.build() {
                Ok(t) => Ok(t),
                Err(e) => Err(Error::TextBuild(e)),
            })?,
            pt,
        );
        Ok(())
    }
}
