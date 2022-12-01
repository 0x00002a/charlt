use std::io::Write;

use anyhow::Result;
use kurbo::{Affine, Point, Rect, Size};
use piet::{RenderContext, Text, TextAlignment, TextLayout, TextLayoutBuilder};

use super::{Error, TextInfo};

pub trait Render {
    type Error: std::error::Error;
    fn render<P: RenderContext>(&self, area: &Rect, r: &mut P) -> Result<(), Self::Error>;
}

pub trait RenderContextExt {
    type TextLayout: TextLayout;
    fn render_text<P: Into<Point>>(
        &mut self,
        pt: P,
        info: &TextInfo,
    ) -> Result<Self::TextLayout, Error>;
    fn text_bounds(&mut self, txt: &TextInfo) -> Result<Size, Error>;
    fn make_text_layout(&mut self, txt: &TextInfo) -> Result<Self::TextLayout, Error>;
}

impl<R: RenderContext> RenderContextExt for R {
    type TextLayout = R::TextLayout;
    fn render_text<P: Into<Point>>(
        &mut self,
        pti: P,
        info: &TextInfo,
    ) -> Result<R::TextLayout, Error> {
        let pt = pti.into();
        let txt = self.make_text_layout(info)?;
        let align = match info.alignment {
            Some(TextAlignment::Center) => Affine::translate(-(txt.size() / 2.0).to_vec2()),
            Some(TextAlignment::End) => {
                Affine::translate((-txt.size().width, -txt.size().height / 2.0))
            }
            _ => Affine::IDENTITY,
        };
        self.save()?;
        let to_pt = Affine::translate(pt.to_vec2());
        self.transform(to_pt * align * info.affine * to_pt.inverse());
        self.draw_text(&txt, pt);
        self.restore()?;
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
