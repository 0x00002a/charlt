use super::{Draw, Entity};

struct Doc {
    nodes: Vec<Node>,
}
impl Doc {
    fn add_node(self, n: Node) -> Self {
        self.nodes.push(n);
        self
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
    fn attr<N: ToString, V: ToString>(self, name: N, value: V) -> Self {
        self.attrs.push((name.to_string(), value.to_string()));
        self
    }
    fn child(self, c: Node) -> Self {
        self.children.push(c);
        self
    }
}

#[allow(non_upper_case_globals)]
mod element {
    use super::*;
    pub const Path: Element = Element::new("Path");
    pub const Line: Element = Element::new("line");
    pub const Rect: Element = Element::new("rect");
}

struct Svg {
    doc: Doc,
}
trait ToSvg {
    fn to_svg(&self) -> Node;
}

impl ToSvg for geo::Geometry {
    fn to_svg(&self) -> Node {
        Node::Element(match &self {
            geo::Geometry::Point(_) => todo!(),
            geo::Geometry::Line(l) => element::Line
                .attr("x1", l.start.x.to_string())
                .attr("y1", l.start.y)
                .attr("x2", l.end.x)
                .attr("y2", l.end.y),
            geo::Geometry::LineString(_) => todo!(),
            geo::Geometry::Polygon(_) => todo!(),
            geo::Geometry::MultiPoint(_) => todo!(),
            geo::Geometry::MultiLineString(_) => todo!(),
            geo::Geometry::MultiPolygon(_) => todo!(),
            geo::Geometry::GeometryCollection(_) => todo!(),
            geo::Geometry::Rect(r) => element::Rect
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
            Node::Element(el.attr("style", format!("color: '{}'", self.colour.to_hex())))
        } else {
            unreachable!()
        }
    }
}

impl Draw for Svg {
    fn draw(self, ent: Entity) -> Self {
        self.doc = self.doc.add_node(ent.to_svg());
        self
    }

    fn dump<W: std::io::Write>(&self, out: &mut W) {}
}
impl ToString for Doc {
    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" standalone=\"no\"?>\n");
        let inner = self
            .nodes
            .iter()
            .fold(Element::new("svg"), |e, n| e.child(n.clone()));
        out.push_str(&inner.to_string());
        out
    }
}

impl ToString for Element {
    fn to_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("<{}>", self.name));
        out.push_str(
            &self
                .attrs
                .iter()
                .map(|(n, v)| format!("{}=\"{}\"", n, v))
                .fold("".to_owned(), |xs, x| xs + " " + &x),
        );
        out.push_str(
            &self
                .children
                .iter()
                .map(|c| c.to_string())
                .reduce(|xs, x| xs + "\n" + &x)
                .unwrap_or("".to_owned()),
        );
        if self.children.len() > 0 {
            out.push_str(&format!("\n</{}>", self.name));
        } else {
            out.push_str("/>");
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
