use super::ffi;
use crate::geom;
use autocxx::prelude::*;
use std::pin::Pin;

pub struct Vertex(pub(crate) Pin<Box<ffi::occara::shape::Vertex>>);

impl Vertex {
    #[must_use]
    pub fn new() -> Self {
        Self(ffi::occara::shape::Vertex::new(0.0, 0.0, 0.0).within_box())
    }

    pub fn set_coordinates(&mut self, x: f64, y: f64, z: f64) {
        self.0.as_mut().set_coordinates(x, y, z);
    }

    #[must_use]
    pub fn get_coordinates(&self) -> (f64, f64, f64) {
        let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
        self.0
            .get_coordinates(Pin::new(&mut x), Pin::new(&mut y), Pin::new(&mut z));
        (x, y, z)
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Shape(pub(crate) Pin<Box<ffi::occara::shape::Shape>>);

impl Shape {
    #[must_use]
    pub fn make_fillet(&self) -> FilletBuilder {
        FilletBuilder(ffi::occara::shape::Shape::make_fillet(&self.0).within_box())
    }

    #[must_use]
    pub fn edges(&self) -> EdgeIterator {
        EdgeIterator(ffi::occara::shape::EdgeIterator::new(&self.0).within_box())
    }
}

pub struct EdgeIterator(pub(crate) Pin<Box<ffi::occara::shape::EdgeIterator>>);

impl Iterator for EdgeIterator {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        let edge_iterator = self.0.as_mut();
        if edge_iterator.more() {
            Some(Edge(edge_iterator.next().within_box()))
        } else {
            None
        }
    }
}

pub struct FilletBuilder(pub(crate) Pin<Box<ffi::occara::shape::FilletBuilder>>);

impl FilletBuilder {
    pub fn add(&mut self, radius: f64, edge: &Edge) {
        self.0.as_mut().add_edge(radius, &edge.0);
    }
    #[must_use]
    pub fn build(&mut self) -> Shape {
        Shape(self.0.as_mut().build().within_box())
    }
}

pub struct Edge(pub(crate) Pin<Box<ffi::occara::shape::Edge>>);

impl Edge {
    #[must_use]
    pub fn arc_of_circle(p1: &geom::Point, p2: &geom::Point, p3: &geom::Point) -> Self {
        geom::TrimmedCurve::arc_of_circle(p1, p2, p3).into()
    }

    #[must_use]
    pub fn line(p1: &geom::Point, p2: &geom::Point) -> Self {
        geom::TrimmedCurve::line(p1, p2).into()
    }
}

impl From<geom::TrimmedCurve> for Edge {
    fn from(curve: geom::TrimmedCurve) -> Self {
        Self(ffi::occara::shape::Edge::new(&curve.0).within_box())
    }
}

pub struct Face(pub(crate) Pin<Box<ffi::occara::shape::Face>>);

impl Face {
    #[must_use]
    pub fn extrude(&self, vec: &geom::Vector) -> Shape {
        Shape(self.0.extrude(&vec.0).within_box())
    }
}

pub struct Wire(pub(crate) Pin<Box<ffi::occara::shape::Wire>>);

impl Wire {
    #[must_use]
    pub fn new(edges: &[&dyn AddableToWire]) -> Self {
        moveit! {
            let mut w = ffi::occara::shape::MakeWire::new();
        }
        for edge in edges {
            edge.add_to_wire(w.as_mut());
        }
        Self(ffi::occara::shape::Wire::new(w.as_mut()).within_box())
    }

    #[must_use]
    pub fn make_face(&self) -> Face {
        Face(self.0.make_face().within_box())
    }
}

impl Clone for Wire {
    fn clone(&self) -> Self {
        Self(ffi::occara::shape::Wire::clone(&self.0).within_box())
    }
}

pub trait AddableToWire {
    fn add_to_wire(&self, maker: Pin<&mut ffi::occara::shape::MakeWire>);
}

impl AddableToWire for Edge {
    fn add_to_wire(&self, mut maker: Pin<&mut ffi::occara::shape::MakeWire>) {
        maker.as_mut().add_edge(&self.0);
    }
}

impl AddableToWire for Wire {
    fn add_to_wire(&self, mut maker: Pin<&mut ffi::occara::shape::MakeWire>) {
        maker.as_mut().add_wire(&self.0);
    }
}

pub fn make_cylinder(axis: &geom::Axis2d, radius: f64, height: f64) -> Shape {
    Shape(ffi::occara::shape::make_cylinder(&axis.0.as_ref(), radius, height).within_box())
}
