use std::io::Write;

use anyhow::Result;
use kurbo::{Rect, Size};
use piet::{RenderContext, Text, TextAlignment, TextLayout, TextLayoutBuilder};

use super::{Error, TextInfo};

pub trait Render {
    type Error: std::error::Error;
    fn render<P: RenderContext>(&self, area: &Rect, r: &mut P) -> Result<(), Self::Error>;
}

pub trait RenderContextExt {
    type TextLayout: TextLayout;
    fn render_text(&mut self, pt: kurbo::Point, info: &TextInfo)
        -> Result<Self::TextLayout, Error>;
    fn text_bounds(&mut self, txt: &TextInfo) -> Result<Size, Error>;
    fn make_text_layout(&mut self, txt: &TextInfo) -> Result<Self::TextLayout, Error>;
}

impl<R: RenderContext> RenderContextExt for R {
    type TextLayout = R::TextLayout;
    fn render_text(
        &mut self,
        mut pt: kurbo::Point,
        info: &TextInfo,
    ) -> Result<R::TextLayout, Error> {
        let txt = self.make_text_layout(info)?;
        match info.alignment {
            Some(TextAlignment::Center) => {
                pt -= (txt.size() / 2.0).to_vec2();
            }
            _ => (),
        }
        self.draw_text(&txt, pt);
        Ok(txt)
    }

    fn text_bounds(&mut self, txt: &TextInfo) -> Result<Size, Error> {
        Ok(self.make_text_layout(txt)?.size())
    }

    fn make_text_layout(&mut self, info: &TextInfo) -> Result<Self::TextLayout, Error> {
        let mut t = self.text().new_text_layout(info.content.to_owned());
        if let Some(f) = &info.font {
            t = t.font(f.family.clone().to_family(self)?, f.size);
        }
        if let Some(c) = info.colour {
            t = t.text_color(c);
        }
        if let Some(a) = info.alignment {
            t = t.alignment(a);
        }
        let txt = (match t.build() {
            Ok(t) => Ok(t),
            Err(e) => Err(Error::TextBuild(e)),
        })?;
        Ok(txt)
    }
}
