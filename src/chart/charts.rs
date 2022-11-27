use crate::render::{Entity, Render};

use super::ChartType;

pub enum Charts {
    XYScatter(Chart<XYScatter>),
    Bar(),
}

struct XYPoint {
    x: f64,
    y: f64,
}
impl From<XYPoint> for geo::Coord {
    fn from(me: XYPoint) -> Self {
        geo::Coord { x: me.x, y: me.y }
    }
}

type BarPoint = f64;
struct BarChart {}
impl ChartType for BarChart {
    type DataPoint = BarPoint;

    const NAME: &'static str = "bar";

    fn render_datasets(&self, series: &Vec<Self::DataPoint>) -> Vec<geo::Geometry> {
        todo!()
    }
}

struct XYScatter {}

impl ChartType for XYScatter {
    type DataPoint = XYPoint;
    const NAME: &'static str = "xy-scatter";

    fn render_datasets<P: crate::utils::Holds<Item = Vec<Self::DataPoint>>>(
        &self,
        datasets: &Vec<P>,
    ) -> Vec<P> {
        datasets
            .iter()
            .map(|p| {
                p.map(|sets| {
                    let mut out = Vec::new();
                    for n in 1..out.len() {
                        let curr_pt = sets[n];
                        let last_pt = sets[n - 1];
                        let pt = geo::Line::new(last_pt, curr_pt);
                        out.push(pt.into());
                    }
                    out
                })
            })
            .collect()
    }

    /*fn render_datasets<P:  (&self, sets: &Vec<Self::DataPoint>) -> Vec<geo::Geometry> {
        let mut out = Vec::new();
        for n in 1..out.len() {
            let curr_pt = sets[n];
            let last_pt = sets[n - 1];
            let pt = geo::Line::new(last_pt, curr_pt);
            out.push(pt.into());
        }
        out
    }*/
}
