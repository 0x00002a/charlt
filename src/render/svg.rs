use super::{Draw, Entity, Shape};
use piet_svg::Text;

struct Doc {
    nodes: Vec<Node>,
    width: u32,
    height: u32,
}
impl Doc {
    fn new(width: u32, height: u32) -> Self {
        Self {
            nodes: Vec::new(),
            width,
            height,
        }
    }
    fn add_node(&mut self, n: Node) {
        self.nodes.push(n);
    }
    fn render(&self, header: bool) -> String {
        let mut out = String::new();
        if header {
            out.push_str("<?xml version=\"1.0\" standalone=\"no\"?>\n");
        }
        let inner = self.nodes.iter().fold(
            Element::new("svg")
                .attr("width", self.width)
                .attr("height", self.height),
            |e, n| e.child(n.clone()),
        );
        out.push_str(&inner.to_string());
        out
    }
}

#[derive(Clone, Debug)]
enum Node {
    Element(Element),
    Text(String),
}
#[derive(Clone, Debug)]
struct Element {
    attrs: Vec<(String, String)>,
    name: String,
    children: Vec<Node>,
}
impl Element {
    fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            attrs: Vec::new(),
            children: Vec::new(),
        }
    }
    fn attr<N: ToString, V: ToString>(mut self, name: N, value: V) -> Self {
        self.attrs.push((name.to_string(), value.to_string()));
        self
    }

    fn maybe_attr<N: ToString, V: ToString>(mut self, name: N, value: Option<V>) -> Self {
        if value.is_some() {
            self.attr(name, value.unwrap())
        } else {
            self
        }
    }
    fn child(mut self, c: Node) -> Self {
        self.children.push(c);
        self
    }
}

#[allow(non_upper_case_globals)]
mod element {
    use super::*;
    pub(super) fn path() -> Element {
        Element::new("path")
    }
    pub(super) fn line() -> Element {
        Element::new("line")
    }
    pub(super) fn rect() -> Element {
        Element::new("rect")
    }

    pub(super) fn group() -> Element {
        Element::new("g")
    }

    pub(super) fn text() -> Element {
        Element::new("text")
    }
}

pub struct Svg {
    doc: Doc,
}
impl Svg {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            doc: Doc::new(width, height),
        }
    }
}

trait ToSvg {
    fn to_svg(&self) -> Node;
}

impl ToSvg for geo::Geometry {
    fn to_svg(&self) -> Node {
        Node::Element(match &self {
            geo::Geometry::Point(_) => todo!(),
            geo::Geometry::Line(l) => element::line()
                .attr("x1", l.start.x.to_string())
                .attr("y1", l.start.y)
                .attr("x2", l.end.x)
                .attr("y2", l.end.y),
            geo::Geometry::LineString(_) => todo!(),
            geo::Geometry::Polygon(_) => todo!(),
            geo::Geometry::MultiPoint(_) => todo!(),
            geo::Geometry::MultiLineString(_) => todo!(),
            geo::Geometry::MultiPolygon(_) => todo!(),
            geo::Geometry::GeometryCollection(c) => {
                c.iter().fold(element::group(), |g, e| g.child(e.to_svg()))
            }
            geo::Geometry::Rect(r) => element::rect()
                .attr("x", r.min().x)
                .attr("y", r.min().y)
                .attr("width", r.width())
                .attr("height", r.height()),
            geo::Geometry::Triangle(_) => todo!(),
        })
    }
}
impl ToSvg for Entity {
    fn to_svg(&self) -> Node {
        if let Node::Element(el) = self.shape.to_svg() {
            Node::Element(el.attr("stroke", self.colour.to_hex()))
        } else {
            unreachable!()
        }
    }
}
impl ToSvg for Shape {
    fn to_svg(&self) -> Node {
        match &self {
            Shape::Geo(g) => g.to_svg(),
            Shape::Text {
                pos,
                content,
                rotation,
                font,
            } => Node::Element(
                element::text()
                    .child(Node::Text(content.clone()))
                    .attr(
                        "transform",
                        format!(
                            "translate({} {}) rotate({})",
                            pos.x,
                            pos.y,
                            rotation.unwrap_or(0.0)
                        ),
                    )
                    .attr("text-anchor", "middle"), //.attr("style", format!("font-family: {};", font.full_name())),
            ),
        }
    }
}

impl Draw for Svg {
    fn draw(&mut self, ent: Entity) {
        self.doc.add_node(ent.to_svg());
    }

    fn dump<W: std::io::Write>(&self, out: &mut W) -> anyhow::Result<()> {
        out.write_all(&self.doc.to_string().as_bytes())?;
        Ok(())
    }
}
impl ToString for Doc {
    fn to_string(&self) -> String {
        self.render(true)
    }
}

impl ToString for Element {
    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("<{} ", self.name));
        out.push_str(
            &self
                .attrs
                .iter()
                .map(|(n, v)| format!("{}=\"{}\"", n, v))
                .fold("".to_owned(), |xs, x| xs + " " + &x),
        );
        if self.children.len() > 0 {
            out.push_str(">\n")
        }
        out.push_str(
            &self
                .children
                .iter()
                .map(|c| c.to_string())
                .reduce(|xs, x| xs + "\n" + &x)
                .unwrap_or("/>".to_owned()),
        );
        if self.children.len() > 0 {
            out.push_str(&format!("\n</{}>", self.name));
        }
        out
    }
}

impl ToString for Node {
    fn to_string(&self) -> String {
        match &self {
            Node::Element(e) => e.to_string(),
            Node::Text(t) => t.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_el_to_node() {
        let mut d = Doc::new(10, 10);
        d.add_node(Node::Element(Element::new("e")));
        let expected = r#"<svg >
<e />
</svg>"#
            .to_owned();
        assert_eq!(d.render(false), expected);
    }
}
