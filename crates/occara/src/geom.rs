//! Geometry module for points, vectors, directions, and surfaces.
//!
//! # Panics
//! Functions in this module may panic if the underlying C++ FFI layer returns null pointers,
//! which would indicate a bug in the C++ bindings. In normal operation, these panics should never occur.

#![allow(clippy::missing_panics_doc)]
#![allow(clippy::fallible_impl_from)]

use crate::ffi;
use std::fmt;

pub struct Point(pub(crate) cxx::UniquePtr<ffi::Point>);

impl Point {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi::Point_create(x, y, z))
    }

    #[must_use]
    pub fn x(&self) -> f64 {
        ffi::Point_x(&self.0)
    }

    #[must_use]
    pub fn y(&self) -> f64 {
        ffi::Point_y(&self.0)
    }

    #[must_use]
    pub fn z(&self) -> f64 {
        ffi::Point_z(&self.0)
    }

    #[must_use]
    pub fn get_coordinates(&self) -> (f64, f64, f64) {
        let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
        ffi::Point_get_coordinates(&self.0, &mut x, &mut y, &mut z);
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
        Self(ffi::Point_clone(&self.0))
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.get_coordinates();
        write!(f, "Point({x:.6}, {y:.6}, {z:.6})")
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.get_coordinates();
        f.debug_struct("Point")
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

pub struct Direction(pub(crate) cxx::UniquePtr<ffi::Direction>);

impl Direction {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi::Direction_create(x, y, z))
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
        ffi::Direction_get_components(&self.0, &mut x, &mut y, &mut z);
        (x, y, z)
    }
}

impl Clone for Direction {
    fn clone(&self) -> Self {
        Self(ffi::Direction_clone(&self.0))
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

pub struct Axis(pub(crate) cxx::UniquePtr<ffi::Axis>);

impl Axis {
    #[must_use]
    pub fn new(location: &Point, direction: &Direction) -> Self {
        Self(ffi::Axis_create(&location.0, &direction.0))
    }

    #[must_use]
    pub fn location(&self) -> Point {
        Point(ffi::Axis_location(&self.0))
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        Direction(ffi::Axis_direction(&self.0))
    }
}

impl Clone for Axis {
    fn clone(&self) -> Self {
        Self(ffi::Axis_clone(&self.0))
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

pub struct Point2D(pub(crate) cxx::UniquePtr<ffi::Point2D>);

impl Point2D {
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self(ffi::Point2D_create(x, y))
    }

    #[must_use]
    pub fn x(self) -> f64 {
        ffi::Point2D_x(&self.0)
    }

    #[must_use]
    pub fn y(self) -> f64 {
        ffi::Point2D_y(&self.0)
    }

    #[must_use]
    pub fn get_coordinates(&self) -> (f64, f64) {
        let (mut x, mut y) = (0.0, 0.0);
        ffi::Point2D_get_coordinates(&self.0, &mut x, &mut y);
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
        Self(ffi::Point2D_clone(&self.0))
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

pub struct Direction2D(pub(crate) cxx::UniquePtr<ffi::Direction2D>);

impl Direction2D {
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self(ffi::Direction2D_create(x, y))
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
        ffi::Direction2D_get_components(&self.0, &mut x, &mut y);
        (x, y)
    }
}

impl Clone for Direction2D {
    fn clone(&self) -> Self {
        Self(ffi::Direction2D_clone(&self.0))
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

pub struct Axis2D(pub(crate) cxx::UniquePtr<ffi::Axis2D>);

impl Axis2D {
    #[must_use]
    pub fn new(location: &Point2D, direction: &Direction2D) -> Self {
        Self(ffi::Axis2D_create(&location.0, &direction.0))
    }

    #[must_use]
    pub fn location(&self) -> Point2D {
        Point2D(ffi::Axis2D_location(&self.0))
    }

    #[must_use]
    pub fn direction(&self) -> Direction2D {
        Direction2D(ffi::Axis2D_direction(&self.0))
    }
}

impl Clone for Axis2D {
    fn clone(&self) -> Self {
        Self(ffi::Axis2D_clone(&self.0))
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

pub struct PlaneAxis(pub(crate) cxx::UniquePtr<ffi::PlaneAxis>);

impl PlaneAxis {
    #[must_use]
    pub fn new(location: &Point, direction: &Direction) -> Self {
        Self(ffi::PlaneAxis_create(&location.0, &direction.0))
    }

    #[must_use]
    pub fn location(&self) -> Point {
        Point(ffi::PlaneAxis_location(&self.0))
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        Direction(ffi::PlaneAxis_direction(&self.0))
    }
}

impl Clone for PlaneAxis {
    fn clone(&self) -> Self {
        Self(ffi::PlaneAxis_clone(&self.0))
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

pub struct SpaceAxis(pub(crate) cxx::UniquePtr<ffi::SpaceAxis>);

impl SpaceAxis {
    #[must_use]
    pub fn new(location: &Point, direction: &Direction) -> Self {
        Self(ffi::SpaceAxis_create(&location.0, &direction.0))
    }

    #[must_use]
    pub fn location(&self) -> Point {
        Point(ffi::SpaceAxis_location(&self.0))
    }

    #[must_use]
    pub fn direction(&self) -> Direction {
        Direction(ffi::SpaceAxis_direction(&self.0))
    }
}

impl Clone for SpaceAxis {
    fn clone(&self) -> Self {
        Self(ffi::SpaceAxis_clone(&self.0))
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

pub struct TrimmedCurve(pub(crate) cxx::UniquePtr<ffi::TrimmedCurve>);

impl TrimmedCurve {
    #[must_use]
    pub fn arc_of_circle(p1: &Point, p2: &Point, p3: &Point) -> Self {
        Self(ffi::TrimmedCurve_arc_of_circle(&p1.0, &p2.0, &p3.0))
    }

    #[must_use]
    pub fn line(p1: &Point, p2: &Point) -> Self {
        Self(ffi::TrimmedCurve_line(&p1.0, &p2.0))
    }
}

impl Clone for TrimmedCurve {
    fn clone(&self) -> Self {
        Self(ffi::TrimmedCurve_clone(&self.0))
    }
}

pub struct TrimmedCurve2D(pub(crate) cxx::UniquePtr<ffi::TrimmedCurve2D>);

impl TrimmedCurve2D {
    #[must_use]
    pub fn line(p1: &Point2D, p2: &Point2D) -> Self {
        Self(ffi::TrimmedCurve2D_line(&p1.0, &p2.0))
    }
}

impl Clone for TrimmedCurve2D {
    fn clone(&self) -> Self {
        Self(ffi::TrimmedCurve2D_clone(&self.0))
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

pub struct Curve2D(pub(crate) cxx::UniquePtr<ffi::Curve2D>);

impl Curve2D {
    #[must_use]
    pub fn trim(&self, u1: f64, u2: f64) -> TrimmedCurve2D {
        TrimmedCurve2D(ffi::Curve2D_trim(&self.0, u1, u2))
    }
}

impl From<&TrimmedCurve2D> for Curve2D {
    fn from(curve: &TrimmedCurve2D) -> Self {
        Self(ffi::Curve2D_from_trimmed_curve2d(&curve.0))
    }
}

impl Clone for Curve2D {
    fn clone(&self) -> Self {
        Self(ffi::Curve2D_clone(&self.0))
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

pub struct Ellipse2D(pub(crate) cxx::UniquePtr<ffi::Ellipse2D>);

impl Ellipse2D {
    #[must_use]
    pub fn new(axis: &Axis2D, major_radius: f64, minor_radius: f64) -> Self {
        Self(ffi::Ellipse2D_create(&axis.0, major_radius, minor_radius))
    }

    #[must_use]
    pub fn value(&self, u: f64) -> Point2D {
        Point2D(ffi::Ellipse2D_value(&self.0, u))
    }

    #[must_use]
    pub fn curve(&self) -> Curve2D {
        Curve2D(ffi::Ellipse2D_curve(&self.0))
    }
}

impl Clone for Ellipse2D {
    fn clone(&self) -> Self {
        Self(ffi::Ellipse2D_clone(&self.0))
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

pub struct Plane(pub(crate) cxx::UniquePtr<ffi::Plane>);

impl Plane {
    #[must_use]
    pub fn location(&self) -> Point {
        Point(ffi::Plane_location(&self.0))
    }
}

impl Clone for Plane {
    fn clone(&self) -> Self {
        Self(ffi::Plane_clone(&self.0))
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

pub struct Surface(pub(crate) cxx::UniquePtr<ffi::Surface>);

impl From<&CylindricalSurface> for Surface {
    fn from(cylindrical_surface: &CylindricalSurface) -> Self {
        Self(ffi::Surface_from_cylindrical_surface(
            &cylindrical_surface.0,
        ))
    }
}

impl Surface {
    #[must_use]
    pub fn as_plane(&self) -> Option<Plane> {
        if ffi::Surface_is_plane(&self.0) {
            Some(Plane(ffi::Surface_as_plane(&self.0)))
        } else {
            None
        }
    }
}

impl Clone for Surface {
    fn clone(&self) -> Self {
        Self(ffi::Surface_clone(&self.0))
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

pub struct Transformation(pub(crate) cxx::UniquePtr<ffi::Transformation>);

impl Transformation {
    #[must_use]
    pub fn mirror(axis: &Axis) -> Self {
        let mut transformation = ffi::Transformation_new();
        ffi::Transformation_mirror(transformation.pin_mut(), &axis.0);
        Self(transformation)
    }

    #[must_use]
    pub fn apply<T: Transformable>(&self, object: &T) -> T {
        object.transform(self)
    }
}

impl Clone for Transformation {
    fn clone(&self) -> Self {
        Self(ffi::Transformation_clone(&self.0))
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

pub struct Vector(pub(crate) cxx::UniquePtr<ffi::Vector>);

impl Vector {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(ffi::Vector_create(x, y, z))
    }
}

impl Clone for Vector {
    fn clone(&self) -> Self {
        Self(ffi::Vector_clone(&self.0))
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

pub struct CylindricalSurface(pub(crate) cxx::UniquePtr<ffi::CylindricalSurface>);

impl CylindricalSurface {
    #[must_use]
    pub fn new(plane: &PlaneAxis, radius: f64) -> Self {
        Self(ffi::CylindricalSurface_create(&plane.0, radius))
    }
}

impl Clone for CylindricalSurface {
    fn clone(&self) -> Self {
        Self(ffi::CylindricalSurface_clone(&self.0))
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
