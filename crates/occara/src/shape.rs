use super::ffi::occara::shape as ffi_shape;
use crate::geom;
use autocxx::prelude::*;
use std::pin::Pin;

pub struct Vertex(pub(crate) Pin<Box<ffi_shape::Vertex>>);

impl Vertex {
    #[must_use]
    pub fn from_point(point: &geom::Point) -> Self {
        Self(ffi_shape::Vertex::create(&point.0.as_ref()).within_box())
    }

    #[must_use]
    pub fn point(&self) -> geom::Point {
        geom::Point(self.0.point().within_box())
    }

    #[must_use]
    pub fn get_coordinates(&self) -> (f64, f64, f64) {
        self.point().get_coordinates()
    }

    #[must_use]
    pub fn x(&self) -> f64 {
        self.point().x()
    }

    #[must_use]
    pub fn y(&self) -> f64 {
        self.point().y()
    }

    #[must_use]
    pub fn z(&self) -> f64 {
        self.point().z()
    }
}

impl Clone for Vertex {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct Shape(pub(crate) Pin<Box<ffi_shape::Shape>>);

impl Shape {
    #[must_use]
    pub fn fillet(&self) -> FilletBuilder {
        FilletBuilder(ffi_shape::Shape::fillet(&self.0).within_box())
    }

    #[must_use]
    pub fn edges(&self) -> EdgeIterator {
        EdgeIterator(ffi_shape::EdgeIterator::create(&self.0).within_box())
    }

    #[must_use]
    pub fn faces(&self) -> FaceIterator {
        FaceIterator(ffi_shape::FaceIterator::create(&self.0).within_box())
    }

    #[must_use]
    pub fn fuse(&self, other: &Self) -> Self {
        Self(self.0.fuse(&other.0).within_box())
    }

    #[must_use]
    pub fn shell(&self) -> ShellBuilder {
        ShellBuilder(ffi_shape::ShellBuilder::create(&self.0).within_box())
    }

    #[must_use]
    pub fn cylinder(axis: &geom::PlaneAxis, radius: f64, height: f64) -> Self {
        Self(ffi_shape::Shape::cylinder(&axis.0.as_ref(), radius, height).within_box())
    }
}

impl Clone for Shape {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct EdgeIterator(pub(crate) Pin<Box<ffi_shape::EdgeIterator>>);

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

impl Clone for EdgeIterator {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct FaceIterator(pub(crate) Pin<Box<ffi_shape::FaceIterator>>);

impl Iterator for FaceIterator {
    type Item = Face;

    fn next(&mut self) -> Option<Self::Item> {
        let face_iterator = self.0.as_mut();
        if face_iterator.more() {
            Some(Face(face_iterator.next().within_box()))
        } else {
            None
        }
    }
}

impl Clone for FaceIterator {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct FilletBuilder(pub(crate) Pin<Box<ffi_shape::FilletBuilder>>);

impl FilletBuilder {
    pub fn add(&mut self, radius: f64, edge: &Edge) {
        self.0.as_mut().add_edge(radius, &edge.0);
    }
    #[must_use]
    pub fn build(&mut self) -> Shape {
        Shape(self.0.as_mut().build().within_box())
    }
}

impl Clone for FilletBuilder {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct ShellBuilder(pub(crate) Pin<Box<ffi_shape::ShellBuilder>>);

impl ShellBuilder {
    pub fn faces_to_remove(&mut self, faces: &[&Face]) -> &mut Self {
        for face in faces {
            self.0.as_mut().add_face_to_remove(&face.0);
        }
        self
    }

    pub fn tolerance(&mut self, tolerance: f64) -> &mut Self {
        self.0.as_mut().set_tolerance(tolerance);
        self
    }

    pub fn offset(&mut self, offset: f64) -> &mut Self {
        self.0.as_mut().set_offset(offset);
        self
    }

    pub fn build(&mut self) -> Shape {
        Shape(self.0.as_mut().build().within_box())
    }
}

impl Clone for ShellBuilder {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct Edge(pub(crate) Pin<Box<ffi_shape::Edge>>);

impl Edge {
    #[must_use]
    pub fn arc_of_circle(p1: &geom::Point, p2: &geom::Point, p3: &geom::Point) -> Self {
        geom::TrimmedCurve::arc_of_circle(p1, p2, p3).into()
    }

    #[must_use]
    pub fn line(p1: &geom::Point, p2: &geom::Point) -> Self {
        geom::TrimmedCurve::line(p1, p2).into()
    }

    #[must_use]
    pub fn new_with_surface(curve: &geom::Curve2D, surface: &geom::Surface) -> Self {
        Self(ffi_shape::Edge::from_2d_curve(&curve.0, &surface.0).within_box())
    }
}

impl Clone for Edge {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl From<geom::TrimmedCurve> for Edge {
    fn from(curve: geom::TrimmedCurve) -> Self {
        Self(ffi_shape::Edge::from_curve(&curve.0).within_box())
    }
}

pub struct Face(pub(crate) Pin<Box<ffi_shape::Face>>);

impl Face {
    #[must_use]
    pub fn extrude(&self, vec: &geom::Vector) -> Shape {
        Shape(self.0.extrude(&vec.0).within_box())
    }

    #[must_use]
    pub fn surface(&self) -> geom::Surface {
        geom::Surface(self.0.surface().within_box())
    }
}

impl Clone for Face {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct Wire(pub(crate) Pin<Box<ffi_shape::Wire>>);

impl Wire {
    #[must_use]
    pub fn new(edges: &[&dyn AddableToWire]) -> Self {
        moveit! {
            let mut w = ffi_shape::WireBuilder::new();
        }
        for edge in edges {
            edge.add_to_wire(w.as_mut());
        }
        Self(ffi_shape::Wire::create(w.as_mut()).within_box())
    }

    #[must_use]
    pub fn face(&self) -> Face {
        Face(self.0.face().within_box())
    }

    #[must_use]
    pub fn build_curves_3d(mut self) -> Self {
        ffi_shape::Wire::build_curves_3d(self.0.as_mut());
        self
    }
}

impl Clone for Wire {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl geom::Transformable for Wire {
    fn transform(&self, transformation: &geom::Transformation) -> Self {
        let transformed_wire = self.0.transform(&transformation.0).within_box();
        Self(transformed_wire)
    }
}

pub trait AddableToWire {
    fn add_to_wire(&self, maker: Pin<&mut ffi_shape::WireBuilder>);
}

impl AddableToWire for Edge {
    fn add_to_wire(&self, mut maker: Pin<&mut ffi_shape::WireBuilder>) {
        maker.as_mut().add_edge(&self.0);
    }
}

impl AddableToWire for Wire {
    fn add_to_wire(&self, mut maker: Pin<&mut ffi_shape::WireBuilder>) {
        maker.as_mut().add_wire(&self.0);
    }
}

pub struct Loft(pub(crate) Pin<Box<ffi_shape::Loft>>);

impl Loft {
    #[must_use]
    pub fn new_solid() -> Self {
        let loft = ffi_shape::Loft::create_solid().within_box();
        Self(loft)
    }

    pub fn add_wires(&mut self, wire: &[&Wire]) -> &mut Self {
        for w in wire {
            self.0.as_mut().add_wire(&w.0);
        }
        self
    }

    pub fn ensure_wire_compatibility(&mut self, check: bool) -> &mut Self {
        self.0.as_mut().ensure_wire_compatibility(check);
        self
    }

    pub fn build(&mut self) -> Shape {
        Shape(self.0.as_mut().build().within_box())
    }
}

impl Clone for Loft {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct Compound(pub(crate) Pin<Box<ffi_shape::Compound>>);

impl Default for Compound {
    fn default() -> Self {
        Self::builder()
    }
}

impl Compound {
    #[must_use]
    pub fn builder() -> Self {
        Self(ffi_shape::Compound::new().within_box())
    }

    pub fn add(&mut self, shape: &Shape) -> &mut Self {
        self.0.as_mut().add_shape(&shape.0);
        self
    }

    pub fn build(&mut self) -> Shape {
        Shape(self.0.as_mut().build().within_box())
    }
}
