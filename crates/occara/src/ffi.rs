// This file contains cxx bridge definitions for OpenCASCADE bindings
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_lifetimes)]

#[cxx::bridge]
pub mod ffi {
    // Opaque C++ types from geom namespace
    unsafe extern "C++" {
        include!("cxx_wrapper.hpp");

        #[namespace = "occara::geom"]
        type Point;
        #[namespace = "occara::geom"]
        type Point2D;
        #[namespace = "occara::geom"]
        type Vector;
        #[namespace = "occara::geom"]
        type Direction;
        #[namespace = "occara::geom"]
        type Direction2D;
        #[namespace = "occara::geom"]
        type Axis;
        #[namespace = "occara::geom"]
        type Axis2D;
        #[namespace = "occara::geom"]
        type PlaneAxis;
        #[namespace = "occara::geom"]
        type SpaceAxis;
        #[namespace = "occara::geom"]
        type TrimmedCurve;
        #[namespace = "occara::geom"]
        type TrimmedCurve2D;
        #[namespace = "occara::geom"]
        type Curve2D;
        #[namespace = "occara::geom"]
        type Ellipse2D;
        #[namespace = "occara::geom"]
        type Plane;
        #[namespace = "occara::geom"]
        type Surface;
        #[namespace = "occara::geom"]
        type Transformation;
        #[namespace = "occara::geom"]
        type CylindricalSurface;

        // Point functions
        #[namespace = "occara::geom"]
        fn Point_create(x: f64, y: f64, z: f64) -> UniquePtr<Point>;
        #[namespace = "occara::geom"]
        fn Point_clone(point: &Point) -> UniquePtr<Point>;
        #[namespace = "occara::geom"]
        fn Point_get_coordinates(point: &Point, x: &mut f64, y: &mut f64, z: &mut f64);
        #[namespace = "occara::geom"]
        fn Point_x(point: &Point) -> f64;
        #[namespace = "occara::geom"]
        fn Point_y(point: &Point) -> f64;
        #[namespace = "occara::geom"]
        fn Point_z(point: &Point) -> f64;

        // Point2D functions
        #[namespace = "occara::geom"]
        fn Point2D_create(x: f64, y: f64) -> UniquePtr<Point2D>;
        #[namespace = "occara::geom"]
        fn Point2D_clone(point2d: &Point2D) -> UniquePtr<Point2D>;
        #[namespace = "occara::geom"]
        fn Point2D_get_coordinates(point2d: &Point2D, x: &mut f64, y: &mut f64);
        #[namespace = "occara::geom"]
        fn Point2D_x(point2d: &Point2D) -> f64;
        #[namespace = "occara::geom"]
        fn Point2D_y(point2d: &Point2D) -> f64;

        // Vector functions
        #[namespace = "occara::geom"]
        fn Vector_create(x: f64, y: f64, z: f64) -> UniquePtr<Vector>;
        #[namespace = "occara::geom"]
        fn Vector_clone(vector: &Vector) -> UniquePtr<Vector>;

        // Direction functions
        #[namespace = "occara::geom"]
        fn Direction_create(x: f64, y: f64, z: f64) -> UniquePtr<Direction>;
        #[namespace = "occara::geom"]
        fn Direction_clone(direction: &Direction) -> UniquePtr<Direction>;
        #[namespace = "occara::geom"]
        fn Direction_get_components(direction: &Direction, x: &mut f64, y: &mut f64, z: &mut f64);

        // Direction2D functions
        #[namespace = "occara::geom"]
        fn Direction2D_create(x: f64, y: f64) -> UniquePtr<Direction2D>;
        #[namespace = "occara::geom"]
        fn Direction2D_clone(direction2d: &Direction2D) -> UniquePtr<Direction2D>;
        #[namespace = "occara::geom"]
        fn Direction2D_get_components(direction2d: &Direction2D, x: &mut f64, y: &mut f64);

        // Axis functions
        #[namespace = "occara::geom"]
        fn Axis_create(origin: &Point, direction: &Direction) -> UniquePtr<Axis>;
        #[namespace = "occara::geom"]
        fn Axis_clone(axis: &Axis) -> UniquePtr<Axis>;
        #[namespace = "occara::geom"]
        fn Axis_location(axis: &Axis) -> UniquePtr<Point>;
        #[namespace = "occara::geom"]
        fn Axis_direction(axis: &Axis) -> UniquePtr<Direction>;

        // Axis2D functions
        #[namespace = "occara::geom"]
        fn Axis2D_create(origin: &Point2D, direction: &Direction2D) -> UniquePtr<Axis2D>;
        #[namespace = "occara::geom"]
        fn Axis2D_clone(axis2d: &Axis2D) -> UniquePtr<Axis2D>;
        #[namespace = "occara::geom"]
        fn Axis2D_location(axis2d: &Axis2D) -> UniquePtr<Point2D>;
        #[namespace = "occara::geom"]
        fn Axis2D_direction(axis2d: &Axis2D) -> UniquePtr<Direction2D>;

        // PlaneAxis functions
        #[namespace = "occara::geom"]
        fn PlaneAxis_create(origin: &Point, direction: &Direction) -> UniquePtr<PlaneAxis>;
        #[namespace = "occara::geom"]
        fn PlaneAxis_clone(plane_axis: &PlaneAxis) -> UniquePtr<PlaneAxis>;
        #[namespace = "occara::geom"]
        fn PlaneAxis_location(plane_axis: &PlaneAxis) -> UniquePtr<Point>;
        #[namespace = "occara::geom"]
        fn PlaneAxis_direction(plane_axis: &PlaneAxis) -> UniquePtr<Direction>;

        // SpaceAxis functions
        #[namespace = "occara::geom"]
        fn SpaceAxis_create(origin: &Point, direction: &Direction) -> UniquePtr<SpaceAxis>;
        #[namespace = "occara::geom"]
        fn SpaceAxis_clone(space_axis: &SpaceAxis) -> UniquePtr<SpaceAxis>;
        #[namespace = "occara::geom"]
        fn SpaceAxis_location(space_axis: &SpaceAxis) -> UniquePtr<Point>;
        #[namespace = "occara::geom"]
        fn SpaceAxis_direction(space_axis: &SpaceAxis) -> UniquePtr<Direction>;

        // TrimmedCurve functions
        #[namespace = "occara::geom"]
        fn TrimmedCurve_arc_of_circle(
            p1: &Point,
            p2: &Point,
            p3: &Point,
        ) -> UniquePtr<TrimmedCurve>;
        #[namespace = "occara::geom"]
        fn TrimmedCurve_line(p1: &Point, p2: &Point) -> UniquePtr<TrimmedCurve>;
        #[namespace = "occara::geom"]
        fn TrimmedCurve_clone(curve: &TrimmedCurve) -> UniquePtr<TrimmedCurve>;

        // TrimmedCurve2D functions
        #[namespace = "occara::geom"]
        fn TrimmedCurve2D_line(p1: &Point2D, p2: &Point2D) -> UniquePtr<TrimmedCurve2D>;
        #[namespace = "occara::geom"]
        fn TrimmedCurve2D_clone(curve2d: &TrimmedCurve2D) -> UniquePtr<TrimmedCurve2D>;

        // Curve2D functions
        #[namespace = "occara::geom"]
        fn Curve2D_from_trimmed_curve2d(curve: &TrimmedCurve2D) -> UniquePtr<Curve2D>;
        #[namespace = "occara::geom"]
        fn Curve2D_clone(curve2d: &Curve2D) -> UniquePtr<Curve2D>;
        #[namespace = "occara::geom"]
        fn Curve2D_trim(curve2d: &Curve2D, u1: f64, u2: f64) -> UniquePtr<TrimmedCurve2D>;

        // Ellipse2D functions
        #[namespace = "occara::geom"]
        fn Ellipse2D_create(
            axis: &Axis2D,
            major_radius: f64,
            minor_radius: f64,
        ) -> UniquePtr<Ellipse2D>;
        #[namespace = "occara::geom"]
        fn Ellipse2D_clone(ellipse2d: &Ellipse2D) -> UniquePtr<Ellipse2D>;
        #[namespace = "occara::geom"]
        fn Ellipse2D_value(ellipse2d: &Ellipse2D, u: f64) -> UniquePtr<Point2D>;
        #[namespace = "occara::geom"]
        fn Ellipse2D_curve(ellipse2d: &Ellipse2D) -> UniquePtr<Curve2D>;

        // Plane functions
        #[namespace = "occara::geom"]
        fn Plane_clone(plane: &Plane) -> UniquePtr<Plane>;
        #[namespace = "occara::geom"]
        fn Plane_location(plane: &Plane) -> UniquePtr<Point>;

        // Surface functions
        #[namespace = "occara::geom"]
        fn Surface_from_cylindrical_surface(surface: &CylindricalSurface) -> UniquePtr<Surface>;
        #[namespace = "occara::geom"]
        fn Surface_clone(surface: &Surface) -> UniquePtr<Surface>;
        #[namespace = "occara::geom"]
        fn Surface_is_plane(surface: &Surface) -> bool;
        #[namespace = "occara::geom"]
        fn Surface_as_plane(surface: &Surface) -> UniquePtr<Plane>;

        // Transformation functions
        #[namespace = "occara::geom"]
        fn Transformation_new() -> UniquePtr<Transformation>;
        #[namespace = "occara::geom"]
        fn Transformation_clone(transformation: &Transformation) -> UniquePtr<Transformation>;
        #[namespace = "occara::geom"]
        fn Transformation_mirror(transformation: Pin<&mut Transformation>, axis: &Axis);

        // CylindricalSurface functions
        #[namespace = "occara::geom"]
        fn CylindricalSurface_create(
            axis: &PlaneAxis,
            radius: f64,
        ) -> UniquePtr<CylindricalSurface>;
        #[namespace = "occara::geom"]
        fn CylindricalSurface_clone(
            cylindrical_surface: &CylindricalSurface,
        ) -> UniquePtr<CylindricalSurface>;
    }

    // Opaque C++ types from shape namespace
    // Note: cxx_wrapper.hpp already included above, no need to include shape.hpp again
    unsafe extern "C++" {

        #[namespace = "occara::shape"]
        type Vertex;
        #[namespace = "occara::shape"]
        type FilletBuilder;
        #[namespace = "occara::shape"]
        type ShellBuilder;
        #[namespace = "occara::shape"]
        type Shape;
        #[namespace = "occara::shape"]
        type Edge;
        #[namespace = "occara::shape"]
        type EdgeIterator;
        #[namespace = "occara::shape"]
        type Face;
        #[namespace = "occara::shape"]
        type FaceIterator;
        #[namespace = "occara::shape"]
        type Wire;
        #[namespace = "occara::shape"]
        type WireBuilder;
        #[namespace = "occara::shape"]
        type Loft;
        #[namespace = "occara::shape"]
        type Compound;
        #[namespace = "occara::shape"]
        type Mesh;

        // Vertex functions
        #[namespace = "occara::shape"]
        fn Vertex_create(point: &Point) -> UniquePtr<Vertex>;
        #[namespace = "occara::shape"]
        fn Vertex_clone(vertex: &Vertex) -> UniquePtr<Vertex>;
        #[namespace = "occara::shape"]
        fn Vertex_point(vertex: &Vertex) -> UniquePtr<Point>;

        // FilletBuilder functions
        #[namespace = "occara::shape"]
        fn FilletBuilder_clone(fillet_builder: &FilletBuilder) -> UniquePtr<FilletBuilder>;
        #[namespace = "occara::shape"]
        fn FilletBuilder_add_edge(
            fillet_builder: Pin<&mut FilletBuilder>,
            radius: f64,
            edge: &Edge,
        );
        #[namespace = "occara::shape"]
        fn FilletBuilder_build(fillet_builder: Pin<&mut FilletBuilder>) -> UniquePtr<Shape>;

        // ShellBuilder functions
        #[namespace = "occara::shape"]
        fn ShellBuilder_create(shape: &Shape) -> UniquePtr<ShellBuilder>;
        #[namespace = "occara::shape"]
        fn ShellBuilder_clone(shell_builder: &ShellBuilder) -> UniquePtr<ShellBuilder>;
        #[namespace = "occara::shape"]
        fn ShellBuilder_add_face_to_remove(shell_builder: Pin<&mut ShellBuilder>, face: &Face);
        #[namespace = "occara::shape"]
        fn ShellBuilder_set_offset(shell_builder: Pin<&mut ShellBuilder>, offset: f64);
        #[namespace = "occara::shape"]
        fn ShellBuilder_set_tolerance(shell_builder: Pin<&mut ShellBuilder>, tolerance: f64);
        #[namespace = "occara::shape"]
        fn ShellBuilder_build(shell_builder: Pin<&mut ShellBuilder>) -> UniquePtr<Shape>;

        // Shape functions
        #[namespace = "occara::shape"]
        fn Shape_clone(shape: &Shape) -> UniquePtr<Shape>;
        #[namespace = "occara::shape"]
        fn Shape_fillet(shape: &Shape) -> UniquePtr<FilletBuilder>;
        #[namespace = "occara::shape"]
        fn Shape_fuse(shape: &Shape, other: &Shape) -> UniquePtr<Shape>;
        #[namespace = "occara::shape"]
        fn Shape_subtract(shape: &Shape, other: &Shape) -> UniquePtr<Shape>;
        #[namespace = "occara::shape"]
        fn Shape_intersect(shape: &Shape, other: &Shape) -> UniquePtr<Shape>;
        #[namespace = "occara::shape"]
        fn Shape_cylinder(axis: &PlaneAxis, radius: f64, height: f64) -> UniquePtr<Shape>;
        #[namespace = "occara::shape"]
        fn Shape_mesh(shape: &Shape) -> UniquePtr<Mesh>;
        #[namespace = "occara::shape"]
        fn Shape_shape_type(shape: &Shape) -> u32;
        #[namespace = "occara::shape"]
        fn Shape_is_null(shape: &Shape) -> bool;
        #[namespace = "occara::shape"]
        fn Shape_is_closed(shape: &Shape) -> bool;
        #[namespace = "occara::shape"]
        fn Shape_mass(shape: &Shape) -> f64;

        // Edge functions
        #[namespace = "occara::shape"]
        fn Edge_from_curve(curve: &TrimmedCurve) -> UniquePtr<Edge>;
        #[namespace = "occara::shape"]
        fn Edge_clone(edge: &Edge) -> UniquePtr<Edge>;
        #[namespace = "occara::shape"]
        fn Edge_from_2d_curve(curve: &Curve2D, surface: &Surface) -> UniquePtr<Edge>;

        // EdgeIterator functions
        #[namespace = "occara::shape"]
        fn EdgeIterator_create(shape: &Shape) -> UniquePtr<EdgeIterator>;
        #[namespace = "occara::shape"]
        fn EdgeIterator_clone(edge_iterator: &EdgeIterator) -> UniquePtr<EdgeIterator>;
        #[namespace = "occara::shape"]
        fn EdgeIterator_more(edge_iterator: &EdgeIterator) -> bool;
        #[namespace = "occara::shape"]
        fn EdgeIterator_next(edge_iterator: Pin<&mut EdgeIterator>) -> UniquePtr<Edge>;

        // Face functions
        #[namespace = "occara::shape"]
        fn Face_clone(face: &Face) -> UniquePtr<Face>;
        #[namespace = "occara::shape"]
        fn Face_extrude(face: &Face, vector: &Vector) -> UniquePtr<Shape>;
        #[namespace = "occara::shape"]
        fn Face_surface(face: &Face) -> UniquePtr<Surface>;

        // FaceIterator functions
        #[namespace = "occara::shape"]
        fn FaceIterator_create(shape: &Shape) -> UniquePtr<FaceIterator>;
        #[namespace = "occara::shape"]
        fn FaceIterator_clone(face_iterator: &FaceIterator) -> UniquePtr<FaceIterator>;
        #[namespace = "occara::shape"]
        fn FaceIterator_more(face_iterator: &FaceIterator) -> bool;
        #[namespace = "occara::shape"]
        fn FaceIterator_next(face_iterator: Pin<&mut FaceIterator>) -> UniquePtr<Face>;

        // Wire functions
        #[namespace = "occara::shape"]
        fn Wire_create(make_wire: Pin<&mut WireBuilder>) -> UniquePtr<Wire>;
        #[namespace = "occara::shape"]
        fn Wire_clone(wire: &Wire) -> UniquePtr<Wire>;
        #[namespace = "occara::shape"]
        fn Wire_transform(wire: &Wire, transformation: &Transformation) -> UniquePtr<Wire>;
        #[namespace = "occara::shape"]
        fn Wire_face(wire: &Wire) -> UniquePtr<Face>;
        #[namespace = "occara::shape"]
        fn Wire_build_curves_3d(wire: Pin<&mut Wire>);

        // WireBuilder functions
        #[namespace = "occara::shape"]
        fn WireBuilder_new() -> UniquePtr<WireBuilder>;
        #[namespace = "occara::shape"]
        fn WireBuilder_clone(wire_builder: &WireBuilder) -> UniquePtr<WireBuilder>;
        #[namespace = "occara::shape"]
        fn WireBuilder_add_edge(wire_builder: Pin<&mut WireBuilder>, edge: &Edge);
        #[namespace = "occara::shape"]
        fn WireBuilder_add_wire(wire_builder: Pin<&mut WireBuilder>, wire: &Wire);

        // Loft functions
        #[namespace = "occara::shape"]
        fn Loft_create_solid() -> UniquePtr<Loft>;
        #[namespace = "occara::shape"]
        fn Loft_clone(loft: &Loft) -> UniquePtr<Loft>;
        #[namespace = "occara::shape"]
        fn Loft_add_wire(loft: Pin<&mut Loft>, wire: &Wire);
        #[namespace = "occara::shape"]
        fn Loft_ensure_wire_compatibility(loft: Pin<&mut Loft>, check: bool);
        #[namespace = "occara::shape"]
        fn Loft_build(loft: Pin<&mut Loft>) -> UniquePtr<Shape>;

        // Compound functions
        #[namespace = "occara::shape"]
        fn Compound_new() -> UniquePtr<Compound>;
        #[namespace = "occara::shape"]
        fn Compound_init(compound: Pin<&mut Compound>);
        #[namespace = "occara::shape"]
        fn Compound_add_shape(compound: Pin<&mut Compound>, shape: &Shape);
        #[namespace = "occara::shape"]
        fn Compound_build(compound: Pin<&mut Compound>) -> UniquePtr<Shape>;

        // Mesh functions
        #[namespace = "occara::shape"]
        fn Mesh_indices_size(mesh: &Mesh) -> usize;
        #[namespace = "occara::shape"]
        fn Mesh_vertices_size(mesh: &Mesh) -> usize;
        #[namespace = "occara::shape"]
        fn Mesh_indices_at(mesh: &Mesh, index: usize) -> usize;
        #[namespace = "occara::shape"]
        fn Mesh_vertices_at(mesh: &Mesh, index: usize) -> UniquePtr<Point>;

        // MakeBottle function (in global namespace, returns occara::shape::Shape)
        fn MakeBottle_wrapper(theWidth: f64, theHeight: f64, theThickness: f64)
            -> UniquePtr<Shape>;
    }
}

// ShapeType enum defined separately (not in cxx bridge to avoid conflicts)
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShapeType {
    Compound = 0,
    CompoundSolid = 1,
    Solid = 2,
    Shell = 3,
    Face = 4,
    Wire = 5,
    Edge = 6,
    Vertex = 7,
    Shape = 8,
}

impl From<u32> for ShapeType {
    fn from(value: u32) -> Self {
        match value {
            0 => ShapeType::Compound,
            1 => ShapeType::CompoundSolid,
            2 => ShapeType::Solid,
            3 => ShapeType::Shell,
            4 => ShapeType::Face,
            5 => ShapeType::Wire,
            6 => ShapeType::Edge,
            7 => ShapeType::Vertex,
            8 => ShapeType::Shape,
            _ => panic!("Invalid ShapeType value: {}", value),
        }
    }
}

impl From<ShapeType> for u32 {
    fn from(value: ShapeType) -> Self {
        value as u32
    }
}

// Re-export for backward compatibility with old module structure
pub use ffi::*;

pub mod occara {
    pub mod geom {
        pub use crate::ffi::*;
    }
    pub mod shape {
        pub use super::super::ShapeType;
        pub use crate::ffi::*;
    }
}
