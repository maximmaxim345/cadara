use std::pin::Pin;

use crate::shape::Wire;

use super::ffi;
use autocxx::prelude::*;

pub struct Point(pub(crate) Pin<Box<ffi::occara::geom::Point>>);

impl Point {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi::occara::geom::Point::new(x, y, z).within_box())
    }

    #[must_use]
    pub fn x(self) -> f64 {
        self.0.x()
    }

    #[must_use]
    pub fn y(self) -> f64 {
        self.0.y()
    }

    #[must_use]
    pub fn z(self) -> f64 {
        self.0.z()
    }

    #[must_use]
    pub fn get_coordinates(&self) -> (f64, f64, f64) {
        let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
        self.0
            .get_coordinates(Pin::new(&mut x), Pin::new(&mut y), Pin::new(&mut z));
        (x, y, z)
    }

    #[must_use]
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn axis_with(&self, direction: &Direction) -> Axis {
        Axis::new(self, direction)
    }

    #[must_use]
    pub fn plane_axis_with(&self, direction: &Direction) -> PlaneAxis {
        PlaneAxis::new(self, direction)
    }
}

pub struct Direction(pub(crate) Pin<Box<ffi::occara::geom::Direction>>);

impl Direction {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi::occara::geom::Direction::new(x, y, z).within_box())
    }

    #[must_use]
    pub fn x() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn y() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    #[must_use]
    pub fn z() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }
}

pub struct Axis(pub(crate) Pin<Box<ffi::occara::geom::Axis>>);

impl Axis {
    #[must_use]
    pub fn new(location: &Point, direction: &Direction) -> Self {
        Self(ffi::occara::geom::Axis::new(&location.0, &direction.0).within_box())
    }
}

pub struct Point2D(pub(crate) Pin<Box<ffi::occara::geom::Point2D>>);

impl Point2D {
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self(ffi::occara::geom::Point2D::new(x, y).within_box())
    }

    #[must_use]
    pub fn x(self) -> f64 {
        self.0.x()
    }

    #[must_use]
    pub fn y(self) -> f64 {
        self.0.y()
    }

    #[must_use]
    pub fn get_coordinates(&self) -> (f64, f64) {
        let (mut x, mut y) = (0.0, 0.0);
        self.0.get_coordinates(Pin::new(&mut x), Pin::new(&mut y));
        (x, y)
    }

    #[must_use]
    pub fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    #[must_use]
    pub fn axis2d_with(&self, direction: &Direction2D) -> Axis2D {
        Axis2D::new(self, direction)
    }
}

pub struct Direction2D(pub(crate) Pin<Box<ffi::occara::geom::Direction2D>>);

impl Direction2D {
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self(ffi::occara::geom::Direction2D::new(x, y).within_box())
    }

    #[must_use]
    pub fn x() -> Self {
        Self::new(1.0, 0.0)
    }

    #[must_use]
    pub fn y() -> Self {
        Self::new(0.0, 1.0)
    }
}

pub struct Axis2D(pub(crate) Pin<Box<ffi::occara::geom::Axis2D>>);

impl Axis2D {
    #[must_use]
    pub fn new(location: &Point2D, direction: &Direction2D) -> Self {
        Self(ffi::occara::geom::Axis2D::new(&location.0, &direction.0).within_box())
    }
}

pub struct PlaneAxis(pub(crate) Pin<Box<ffi::occara::geom::PlaneAxis>>);

impl PlaneAxis {
    #[must_use]
    pub fn new(location: &Point, direction: &Direction) -> Self {
        Self(ffi::occara::geom::PlaneAxis::new(&location.0, &direction.0).within_box())
    }
}

// TODO: gp_Ax3 should be SpaceAxis

pub struct TrimmedCurve(pub(crate) Pin<Box<ffi::occara::geom::TrimmedCurve>>);

impl TrimmedCurve {
    #[must_use]
    pub fn arc_of_circle(p1: &Point, p2: &Point, p3: &Point) -> Self {
        Self(ffi::occara::geom::TrimmedCurve::new(&p1.0, &p2.0, &p3.0).within_box())
    }

    #[must_use]
    pub fn line(p1: &Point, p2: &Point) -> Self {
        Self(ffi::occara::geom::TrimmedCurve::new1(&p1.0, &p2.0).within_box())
    }
}

pub struct TrimmedCurve2D(pub(crate) Pin<Box<ffi::occara::geom::TrimmedCurve2D>>);

impl TrimmedCurve2D {
    #[must_use]
    pub fn line(p1: &Point2D, p2: &Point2D) -> Self {
        Self(ffi::occara::geom::TrimmedCurve2D::new1(&p1.0, &p2.0).within_box())
    }
}

pub struct Ellipse2D(pub(crate) Pin<Box<ffi::occara::geom::Ellipse2D>>);

impl Ellipse2D {
    #[must_use]
    pub fn new(axis: &Axis2D, major_radius: f64, minor_radius: f64) -> Self {
        Self(ffi::occara::geom::Ellipse2D::new(&axis.0, major_radius, minor_radius).within_box())
    }

    #[must_use]
    pub fn trim(&self, u1: f64, u2: f64) -> TrimmedCurve2D {
        let trimmed_curve = ffi::occara::geom::Ellipse2D::trim(&self.0, u1, u2).within_box();
        TrimmedCurve2D(trimmed_curve)
    }

    #[must_use]
    pub fn value(&self, u: f64) -> Point2D {
        let point = ffi::occara::geom::Ellipse2D::value(&self.0, u).within_box();
        Point2D(point)
    }
}

pub struct Plane(pub(crate) Pin<Box<ffi::occara::geom::Plane>>);

impl Plane {
    #[must_use]
    pub fn location(&self) -> Point {
        let point = ffi::occara::geom::Plane::location(&self.0).within_box();
        Point(point)
    }
}

pub struct Surface(pub(crate) Pin<Box<ffi::occara::geom::Surface>>);

impl Surface {
    #[must_use]
    pub fn as_plane(&self) -> Option<Plane> {
        if ffi::occara::geom::Surface::is_plane(&self.0) {
            let plane = ffi::occara::geom::Surface::as_plane(&self.0).within_box();
            Some(Plane(plane))
        } else {
            None
        }
    }
}

pub struct Transformation(pub(crate) Pin<Box<ffi::occara::geom::Transformation>>);

impl Transformation {
    #[must_use]
    pub fn mirror(axis: &Axis) -> Self {
        let mut transformation = ffi::occara::geom::Transformation::new().within_box();
        transformation.as_mut().mirror(&axis.0);
        Self(transformation)
    }

    #[must_use]
    pub fn apply(&self, mut wire: Wire) -> Wire {
        let wire = wire.0.as_mut().transform(&self.0).within_box();
        Wire(wire)
    }
}

pub struct Vector(pub(crate) Pin<Box<ffi::occara::geom::Vector>>);

impl Vector {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi::occara::geom::Vector::new(x, y, z).within_box())
    }
}

pub struct CylindricalSurface(pub(crate) Pin<Box<ffi::occara::geom::CylindricalSurface>>);

impl CylindricalSurface {
    #[must_use]
    pub fn new(plane: &PlaneAxis, radius: f64) -> Self {
        Self(ffi::occara::geom::CylindricalSurface::new(&plane.0, radius).within_box())
    }
}
