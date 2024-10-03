use crate::ffi::occara::geom as ffi_geom;
use autocxx::prelude::*;
use std::{fmt, pin::Pin};

pub struct Point(pub(crate) Pin<Box<ffi_geom::Point>>);

impl Point {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi_geom::Point::create(x, y, z).within_box())
    }

    #[must_use]
    pub fn x(&self) -> f64 {
        self.0.x()
    }

    #[must_use]
    pub fn y(&self) -> f64 {
        self.0.y()
    }

    #[must_use]
    pub fn z(&self) -> f64 {
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

impl Clone for Point {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.get_coordinates();
        write!(f, "Vertex({x:.6}, {y:.6}, {z:.6})")
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.get_coordinates();
        f.debug_struct("Vertex")
            .field("x", &x)
            .field("y", &y)
            .field("z", &z)
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Point {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Point {}

pub struct Direction(pub(crate) Pin<Box<ffi_geom::Direction>>);

impl Direction {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi_geom::Direction::create(x, y, z).within_box())
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
    // TODO: maybe rename x/y/z to new_x/new_y/new_z and add x() -> f64

    #[must_use]
    pub fn get_components(&self) -> (f64, f64, f64) {
        let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
        self.0
            .get_components(Pin::new(&mut x), Pin::new(&mut y), Pin::new(&mut z));
        (x, y, z)
    }
}

impl Clone for Direction {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.get_components();
        write!(f, "Vertex({x:.6}, {y:.6}, {z:.6})")
    }
}

impl fmt::Debug for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.get_components();
        f.debug_struct("Direction")
            .field("x", &x)
            .field("y", &y)
            .field("z", &z)
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Direction {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Direction {}

pub struct Axis(pub(crate) Pin<Box<ffi_geom::Axis>>);

impl Axis {
    #[must_use]
    pub fn new(location: &Point, direction: &Direction) -> Self {
        Self(ffi_geom::Axis::create(&location.0, &direction.0).within_box())
    }

    #[must_use]
    pub fn location(&self) -> Point {
        Point(self.0.location().within_box())
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        Direction(self.0.direction().within_box())
    }
}

impl Clone for Axis {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let loc = self.location().get_coordinates();
        let dir = self.direction().get_components();
        write!(
            f,
            "Axis(loc: ({:.2}, {:.2}, {:.2}), dir: ({:.2}, {:.2}, {:.2}))",
            loc.0, loc.1, loc.2, dir.0, dir.1, dir.2,
        )
    }
}

impl fmt::Debug for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Axis")
            .field("location", &self.location())
            .field("direction", &self.direction())
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Axis {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Axis {}

pub struct Point2D(pub(crate) Pin<Box<ffi_geom::Point2D>>);

impl Point2D {
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self(ffi_geom::Point2D::create(x, y).within_box())
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

impl Clone for Point2D {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y) = self.get_coordinates();
        write!(f, "Point2D({x:.6}, {y:.6})")
    }
}

impl fmt::Debug for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y) = self.get_coordinates();
        f.debug_struct("Point2D")
            .field("x", &x)
            .field("y", &y)
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Point2D {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Point2D {}

pub struct Direction2D(pub(crate) Pin<Box<ffi_geom::Direction2D>>);

impl Direction2D {
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self(ffi_geom::Direction2D::create(x, y).within_box())
    }

    #[must_use]
    pub fn x() -> Self {
        Self::new(1.0, 0.0)
    }

    #[must_use]
    pub fn y() -> Self {
        Self::new(0.0, 1.0)
    }

    #[must_use]
    pub fn get_components(&self) -> (f64, f64) {
        let (mut x, mut y) = (0.0, 0.0);
        self.0.get_components(Pin::new(&mut x), Pin::new(&mut y));
        (x, y)
    }
}

impl Clone for Direction2D {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Display for Direction2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y) = self.get_components();
        write!(f, "Direction2D({x:.6}, {y:.6})")
    }
}

impl fmt::Debug for Direction2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y) = self.get_components();
        f.debug_struct("Direction2D")
            .field("x", &x)
            .field("y", &y)
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Direction2D {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Direction2D {}

pub struct Axis2D(pub(crate) Pin<Box<ffi_geom::Axis2D>>);

impl Axis2D {
    #[must_use]
    pub fn new(location: &Point2D, direction: &Direction2D) -> Self {
        Self(ffi_geom::Axis2D::create(&location.0, &direction.0).within_box())
    }

    #[must_use]
    pub fn location(&self) -> Point2D {
        Point2D(self.0.location().within_box())
    }

    #[must_use]
    pub fn direction(&self) -> Direction2D {
        Direction2D(self.0.direction().within_box())
    }
}

impl Clone for Axis2D {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Display for Axis2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let loc = self.location().get_coordinates();
        let dir = self.direction().get_components();
        write!(
            f,
            "Axis2D(loc: ({:.2}, {:.2}), dir: ({:.2}, {:.2}))",
            loc.0, loc.1, dir.0, dir.1,
        )
    }
}

impl fmt::Debug for Axis2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Axis2D")
            .field("location", &self.location())
            .field("direction", &self.direction())
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Axis2D {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Axis2D {}

pub struct PlaneAxis(pub(crate) Pin<Box<ffi_geom::PlaneAxis>>);

impl PlaneAxis {
    #[must_use]
    pub fn new(location: &Point, direction: &Direction) -> Self {
        Self(ffi_geom::PlaneAxis::create(&location.0, &direction.0).within_box())
    }

    #[must_use]
    pub fn location(&self) -> Point {
        Point(self.0.location().within_box())
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        Direction(self.0.direction().within_box())
    }
}

impl Clone for PlaneAxis {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Display for PlaneAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let loc = self.location().get_coordinates();
        let dir = self.direction().get_components();
        write!(
            f,
            "PlaneAxis(loc: ({:.2}, {:.2}, {:.2}), dir: ({:.2}, {:.2}, {:.2}))",
            loc.0, loc.1, loc.2, dir.0, dir.1, dir.2,
        )
    }
}

impl fmt::Debug for PlaneAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlaneAxis")
            .field("location", &self.location())
            .field("direction", &self.direction())
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for PlaneAxis {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for PlaneAxis {}

pub struct SpaceAxis(pub(crate) Pin<Box<ffi_geom::SpaceAxis>>);

impl SpaceAxis {
    #[must_use]
    pub fn new(location: &Point, direction: &Direction) -> Self {
        Self(ffi_geom::SpaceAxis::create(&location.0, &direction.0).within_box())
    }

    #[must_use]
    pub fn location(&self) -> Point {
        Point(self.0.location().within_box())
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        Direction(self.0.direction().within_box())
    }
}

impl Clone for SpaceAxis {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Display for SpaceAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let loc = self.location().get_coordinates();
        let dir = self.direction().get_components();
        write!(
            f,
            "SpaceAxis(loc: ({:.2}, {:.2}, {:.2}), dir: ({:.2}, {:.2}, {:.2}))",
            loc.0, loc.1, loc.2, dir.0, dir.1, dir.2,
        )
    }
}

impl fmt::Debug for SpaceAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpaceAxis")
            .field("location", &self.location())
            .field("direction", &self.direction())
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for SpaceAxis {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for SpaceAxis {}

pub struct TrimmedCurve(pub(crate) Pin<Box<ffi_geom::TrimmedCurve>>);

impl TrimmedCurve {
    #[must_use]
    pub fn arc_of_circle(p1: &Point, p2: &Point, p3: &Point) -> Self {
        Self(ffi_geom::TrimmedCurve::arc_of_circle(&p1.0, &p2.0, &p3.0).within_box())
    }

    #[must_use]
    pub fn line(p1: &Point, p2: &Point) -> Self {
        Self(ffi_geom::TrimmedCurve::line(&p1.0, &p2.0).within_box())
    }
}

impl Clone for TrimmedCurve {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

pub struct TrimmedCurve2D(pub(crate) Pin<Box<ffi_geom::TrimmedCurve2D>>);

impl TrimmedCurve2D {
    #[must_use]
    pub fn line(p1: &Point2D, p2: &Point2D) -> Self {
        Self(ffi_geom::TrimmedCurve2D::line(&p1.0, &p2.0).within_box())
    }
}

impl Clone for TrimmedCurve2D {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Debug for TrimmedCurve2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("TrimmedCurve2D").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for TrimmedCurve2D {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for TrimmedCurve2D {}

pub struct Curve2D(pub(crate) Pin<Box<ffi_geom::Curve2D>>);

impl Curve2D {
    #[must_use]
    pub fn trim(&self, u1: f64, u2: f64) -> TrimmedCurve2D {
        let trimmed_curve = ffi_geom::Curve2D::trim(&self.0, u1, u2).within_box();
        TrimmedCurve2D(trimmed_curve)
    }
}

impl From<&TrimmedCurve2D> for Curve2D {
    fn from(curve: &TrimmedCurve2D) -> Self {
        Self(ffi_geom::Curve2D::from_trimmed_curve2d(&curve.0).within_box())
    }
}

impl Clone for Curve2D {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Debug for Curve2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Curve2D").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Curve2D {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Curve2D {}

pub struct Ellipse2D(pub(crate) Pin<Box<ffi_geom::Ellipse2D>>);

impl Ellipse2D {
    #[must_use]
    pub fn new(axis: &Axis2D, major_radius: f64, minor_radius: f64) -> Self {
        Self(ffi_geom::Ellipse2D::create(&axis.0, major_radius, minor_radius).within_box())
    }

    #[must_use]
    pub fn value(&self, u: f64) -> Point2D {
        let point = ffi_geom::Ellipse2D::value(&self.0, u).within_box();
        Point2D(point)
    }

    #[must_use]
    pub fn curve(&self) -> Curve2D {
        Curve2D(self.0.curve().within_box())
    }
}

impl Clone for Ellipse2D {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Debug for Ellipse2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Ellipse2D").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Ellipse2D {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Ellipse2D {}

pub struct Plane(pub(crate) Pin<Box<ffi_geom::Plane>>);

impl Plane {
    #[must_use]
    pub fn location(&self) -> Point {
        let point = ffi_geom::Plane::location(&self.0).within_box();
        Point(point)
    }
}

impl Clone for Plane {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Debug for Plane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Plane").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Plane {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Plane {}

pub struct Surface(pub(crate) Pin<Box<ffi_geom::Surface>>);

impl From<&CylindricalSurface> for Surface {
    fn from(cylindrical_surface: &CylindricalSurface) -> Self {
        Self(ffi_geom::Surface::from_cylindrical_surface(&cylindrical_surface.0).within_box())
    }
}

impl Surface {
    #[must_use]
    pub fn as_plane(&self) -> Option<Plane> {
        if ffi_geom::Surface::is_plane(&self.0) {
            let plane = ffi_geom::Surface::as_plane(&self.0).within_box();
            Some(Plane(plane))
        } else {
            None
        }
    }
}

impl Clone for Surface {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Debug for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Surface").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Surface {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Surface {}

pub trait Transformable {
    #[must_use]
    fn transform(&self, transformation: &Transformation) -> Self;
}

pub struct Transformation(pub(crate) Pin<Box<ffi_geom::Transformation>>);

impl Transformation {
    #[must_use]
    pub fn mirror(axis: &Axis) -> Self {
        let mut transformation = ffi_geom::Transformation::new().within_box();
        transformation.as_mut().mirror(&axis.0);
        Self(transformation)
    }

    #[must_use]
    pub fn apply<T: Transformable>(&self, object: &T) -> T {
        object.transform(self)
    }
}

impl Clone for Transformation {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Debug for Transformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Transformation").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Transformation {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Transformation {}

pub struct Vector(pub(crate) Pin<Box<ffi_geom::Vector>>);

impl Vector {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi_geom::Vector::create(x, y, z).within_box())
    }
}

impl Clone for Vector {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Debug for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Vector").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Vector {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Vector {}

pub struct CylindricalSurface(pub(crate) Pin<Box<ffi_geom::CylindricalSurface>>);

impl CylindricalSurface {
    #[must_use]
    pub fn new(plane: &PlaneAxis, radius: f64) -> Self {
        Self(ffi_geom::CylindricalSurface::create(&plane.0, radius).within_box())
    }
}

impl Clone for CylindricalSurface {
    fn clone(&self) -> Self {
        Self(self.0.clone().within_box())
    }
}

impl fmt::Debug for CylindricalSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("CylindricalSurface").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for CylindricalSurface {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for CylindricalSurface {}
