use super::ffi::occara::shape as ffi_shape;
use crate::geom;
use autocxx::prelude::*;
use std::pin::Pin;

pub struct Vertex(pub(crate) Pin<Box<ffi_shape::Vertex>>);

impl Vertex {
    #[must_use]
    pub fn new() -> Self {
        Self(ffi_shape::Vertex::new(0.0, 0.0, 0.0).within_box())
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

pub struct Shape(pub(crate) Pin<Box<ffi_shape::Shape>>);

impl Shape {
    #[must_use]
    pub fn fillet(&self) -> FilletBuilder {
        FilletBuilder(ffi_shape::Shape::fillet(&self.0).within_box())
    }

    #[must_use]
    pub fn edges(&self) -> EdgeIterator {
        EdgeIterator(ffi_shape::EdgeIterator::new(&self.0).within_box())
    }

    #[must_use]
    pub fn faces(&self) -> FaceIterator {
        FaceIterator(ffi_shape::FaceIterator::new(&self.0).within_box())
    }

    #[must_use]
    pub fn fuse(&self, other: &Self) -> Self {
        Self(self.0.fuse(&other.0).within_box())
    }

    #[must_use]
    pub fn shell(&self) -> ShellBuilder {
        ShellBuilder(ffi_shape::ShellBuilder::new(&self.0).within_box())
    }

    #[must_use]
    pub fn cylinder(axis: &geom::PlaneAxis, radius: f64, height: f64) -> Self {
        Self(ffi_shape::Shape::cylinder(&axis.0.as_ref(), radius, height).within_box())
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

    // TODO: this should be a geom::Curve2D
    // TODO: this should be a geom::Surface
    #[must_use]
    pub fn new_with_surface(
        curve: &geom::TrimmedCurve2D,
        surface: &geom::CylindricalSurface,
    ) -> Self {
        Self(ffi_shape::Edge::new2(&curve.0, &surface.0).within_box())
    }
}

impl From<geom::TrimmedCurve> for Edge {
    fn from(curve: geom::TrimmedCurve) -> Self {
        Self(ffi_shape::Edge::new(&curve.0).within_box())
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
        Self(ffi_shape::Wire::new(w.as_mut()).within_box())
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
        Self(ffi_shape::Wire::clone(&self.0).within_box())
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
        let loft = ffi_shape::Loft::new(true).within_box();
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

pub struct Compound(pub(crate) Pin<Box<ffi_shape::Compound>>);

impl Default for Compound {
    fn default() -> Self {
        Self::new()
    }
}

impl Compound {
    #[must_use]
    pub fn new() -> Self {
        Self(ffi_shape::Compound::new().within_box())
    }

    pub fn add_shapes(&mut self, shape: &[&Shape]) -> &mut Self {
        for s in shape {
            self.0.as_mut().add_shape(&s.0);
        }
        self
    }

    pub fn build(&mut self) -> Shape {
        Shape(self.0.as_mut().build().within_box())
    }
}
