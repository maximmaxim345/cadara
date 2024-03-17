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
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn axis_with(&self, direction: Direction) -> Axis {
        Axis::new(self, &direction)
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

pub struct Transformation(pub(crate) Pin<Box<ffi::occara::geom::Transformation>>);

impl Transformation {
    #[must_use]
    pub fn mirror(axis: Axis) -> Self {
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
