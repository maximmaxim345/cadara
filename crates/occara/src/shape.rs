//! Shape module for geometric shapes and topological operations.
//!
//! # Panics
//! Functions in this module may panic if the underlying C++ FFI layer returns null pointers,
//! which would indicate a bug in the C++ bindings. In normal operation, these panics should never occur.

#![allow(clippy::missing_panics_doc)]
#![allow(clippy::fallible_impl_from)]

use crate::{ffi, geom};
use std::fmt;
use std::io::Write;
use std::{fs::File, path::Path, pin::Pin};

pub struct Vertex(pub(crate) cxx::UniquePtr<ffi::Vertex>);

impl Vertex {
    #[must_use]
    pub fn from_point(point: &geom::Point) -> Self {
        Self(ffi::Vertex_create(&point.0))
    }

    #[must_use]
    pub fn point(&self) -> geom::Point {
        geom::Point(ffi::Vertex_point(&self.0))
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
        Self(ffi::Vertex_clone(&self.0))
    }
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z) = self.get_coordinates();
        write!(f, "Vertex({x:.6}, {y:.6}, {z:.6})")
    }
}

impl fmt::Debug for Vertex {
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
unsafe impl Send for Vertex {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Vertex {}

pub struct Shape(pub(crate) cxx::UniquePtr<ffi::Shape>);

pub use crate::ffi::ShapeType;

impl ShapeType {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Compound => "Compound",
            Self::CompoundSolid => "CompoundSolid",
            Self::Solid => "Solid",
            Self::Shell => "Shell",
            Self::Face => "Face",
            Self::Wire => "Wire",
            Self::Edge => "Edge",
            Self::Vertex => "Vertex",
            Self::Shape => "Shape",
        }
    }
}

impl Shape {
    #[must_use]
    pub fn fillet(&self) -> FilletBuilder {
        FilletBuilder(ffi::Shape_fillet(&self.0))
    }

    #[must_use]
    pub fn edges(&self) -> EdgeIterator {
        EdgeIterator(ffi::EdgeIterator_create(&self.0))
    }

    #[must_use]
    pub fn faces(&self) -> FaceIterator {
        FaceIterator(ffi::FaceIterator_create(&self.0))
    }

    #[must_use]
    pub fn fuse(&self, other: &Self) -> Self {
        Self(ffi::Shape_fuse(&self.0, &other.0))
    }

    #[must_use]
    pub fn subtract(&self, other: &Self) -> Self {
        Self(ffi::Shape_subtract(&self.0, &other.0))
    }

    #[must_use]
    pub fn intersect(&self, other: &Self) -> Self {
        Self(ffi::Shape_intersect(&self.0, &other.0))
    }

    #[must_use]
    pub fn shell(&self) -> ShellBuilder {
        ShellBuilder(ffi::ShellBuilder_create(&self.0))
    }

    #[must_use]
    pub fn cylinder(axis: &geom::PlaneAxis, radius: f64, height: f64) -> Self {
        Self(ffi::Shape_cylinder(&axis.0, radius, height))
    }

    #[must_use]
    pub fn mesh(&self) -> Mesh {
        Mesh(ffi::Shape_mesh(&self.0))
    }

    #[must_use]
    pub fn shape_type(&self) -> ShapeType {
        ffi::Shape_shape_type(&self.0).into()
    }

    #[must_use]
    pub fn is_null(&self) -> bool {
        ffi::Shape_is_null(&self.0)
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        ffi::Shape_is_closed(&self.0)
    }

    #[must_use]
    pub fn mass(&self) -> f64 {
        ffi::Shape_mass(&self.0)
    }
}

impl Clone for Shape {
    fn clone(&self) -> Self {
        Self(ffi::Shape_clone(&self.0))
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shape({})", self.shape_type().to_str())
    }
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shape")
            .field("type", &self.shape_type().to_str())
            .field("is_null", &self.is_null())
            .field("is_closed", &self.is_closed())
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Shape {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Shape {}

pub struct Mesh(pub(crate) cxx::UniquePtr<ffi::Mesh>);

impl Mesh {
    #[must_use]
    pub fn indices(&self) -> Vec<usize> {
        let mesh = &*self.0;
        let mut indices = Vec::new();
        for i in 0..ffi::Mesh_indices_size(mesh) {
            indices.push(ffi::Mesh_indices_at(mesh, i));
        }
        indices
    }

    #[must_use]
    pub fn vertices(&self) -> Vec<geom::Point> {
        // FIXME: This is a temporary solution, this will be optimized soon
        let mesh = &*self.0;
        let mut vertices = Vec::new();
        for i in 0..ffi::Mesh_vertices_size(mesh) {
            vertices.push(geom::Point(ffi::Mesh_vertices_at(mesh, i)));
        }
        vertices
    }

    /// Export the mesh to an OBJ file
    ///
    /// This function will write the mesh to a file in the Wavefront OBJ format.
    ///
    /// # Arguments
    /// * `path` - The path to the file to write to
    ///
    /// # Errors
    /// This function will return an error if the file cannot be created or written to.
    pub fn export_obj(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let indices = self.indices();
        let vertices = self.vertices();

        let path = path.as_ref();
        let mut file = File::create(path)?;

        // Write vertices
        for vertex in vertices {
            writeln!(file, "v {} {} {}", vertex.x(), vertex.y(), vertex.z())?;
        }

        // Write faces (indices)
        for i in (0..indices.len()).step_by(3) {
            writeln!(
                file,
                "f {} {} {}",
                indices[i] + 1,
                indices[i + 1] + 1,
                indices[i + 2] + 1
            )?;
        }
        Ok(())
    }
}

impl fmt::Debug for Mesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mesh = &*self.0;
        f.debug_struct("Mesh")
            .field("vertices", &ffi::Mesh_vertices_size(mesh))
            .field("indices", &ffi::Mesh_indices_size(mesh))
            .finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Mesh {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Mesh {}

pub struct EdgeIterator(pub(crate) cxx::UniquePtr<ffi::EdgeIterator>);

impl Iterator for EdgeIterator {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we have more elements first
        let has_more = {
            let iter_ref = self.0.as_ref()?;
            ffi::EdgeIterator_more(iter_ref)
        };

        if has_more {
            let edge_ptr = ffi::EdgeIterator_next(self.0.pin_mut());
            // Ensure the edge pointer is valid
            edge_ptr.as_ref()?;
            Some(Edge(edge_ptr))
        } else {
            None
        }
    }
}

impl Clone for EdgeIterator {
    fn clone(&self) -> Self {
        Self(ffi::EdgeIterator_clone(&self.0))
    }
}

impl fmt::Debug for EdgeIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("EdgeIterator").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for EdgeIterator {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for EdgeIterator {}

pub struct FaceIterator(pub(crate) cxx::UniquePtr<ffi::FaceIterator>);

impl Iterator for FaceIterator {
    type Item = Face;

    fn next(&mut self) -> Option<Self::Item> {
        if ffi::FaceIterator_more(&self.0) {
            Some(Face(ffi::FaceIterator_next(self.0.pin_mut())))
        } else {
            None
        }
    }
}

impl Clone for FaceIterator {
    fn clone(&self) -> Self {
        Self(ffi::FaceIterator_clone(&self.0))
    }
}

impl fmt::Debug for FaceIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("FaceIterator").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for FaceIterator {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for FaceIterator {}

pub struct FilletBuilder(pub(crate) cxx::UniquePtr<ffi::FilletBuilder>);

impl FilletBuilder {
    pub fn add(&mut self, radius: f64, edge: &Edge) {
        ffi::FilletBuilder_add_edge(self.0.pin_mut(), radius, &edge.0);
    }
    #[must_use]
    pub fn build(&mut self) -> Shape {
        Shape(ffi::FilletBuilder_build(self.0.pin_mut()))
    }
}

impl Clone for FilletBuilder {
    fn clone(&self) -> Self {
        Self(ffi::FilletBuilder_clone(&self.0))
    }
}

impl fmt::Debug for FilletBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("FilletBuilder").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for FilletBuilder {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for FilletBuilder {}

pub struct ShellBuilder(pub(crate) cxx::UniquePtr<ffi::ShellBuilder>);

impl ShellBuilder {
    pub fn faces_to_remove(&mut self, faces: &[&Face]) -> &mut Self {
        for face in faces {
            ffi::ShellBuilder_add_face_to_remove(self.0.pin_mut(), &face.0);
        }
        self
    }

    pub fn tolerance(&mut self, tolerance: f64) -> &mut Self {
        ffi::ShellBuilder_set_tolerance(self.0.pin_mut(), tolerance);
        self
    }

    pub fn offset(&mut self, offset: f64) -> &mut Self {
        ffi::ShellBuilder_set_offset(self.0.pin_mut(), offset);
        self
    }

    pub fn build(&mut self) -> Shape {
        Shape(ffi::ShellBuilder_build(self.0.pin_mut()))
    }
}

impl Clone for ShellBuilder {
    fn clone(&self) -> Self {
        Self(ffi::ShellBuilder_clone(&self.0))
    }
}

impl fmt::Debug for ShellBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("ShellBuilder").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for ShellBuilder {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for ShellBuilder {}

pub struct Edge(pub(crate) cxx::UniquePtr<ffi::Edge>);

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
        Self(ffi::Edge_from_2d_curve(&curve.0, &surface.0))
    }
}

impl Clone for Edge {
    fn clone(&self) -> Self {
        Self(ffi::Edge_clone(&self.0))
    }
}

impl From<geom::TrimmedCurve> for Edge {
    fn from(curve: geom::TrimmedCurve) -> Self {
        Self(ffi::Edge_from_curve(&curve.0))
    }
}

impl fmt::Debug for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Edge").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Edge {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Edge {}

pub struct Face(pub(crate) cxx::UniquePtr<ffi::Face>);

impl Face {
    #[must_use]
    pub fn extrude(&self, vec: &geom::Vector) -> Shape {
        Shape(ffi::Face_extrude(&self.0, &vec.0))
    }

    #[must_use]
    pub fn surface(&self) -> geom::Surface {
        geom::Surface(ffi::Face_surface(&self.0))
    }
}

impl Clone for Face {
    fn clone(&self) -> Self {
        Self(ffi::Face_clone(&self.0))
    }
}

impl fmt::Debug for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Face").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Face {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Face {}

pub struct Wire(pub(crate) cxx::UniquePtr<ffi::Wire>);

impl Wire {
    #[must_use]
    pub fn new(edges: &[&dyn AddableToWire]) -> Self {
        let mut w = ffi::WireBuilder_new();
        for edge in edges {
            edge.add_to_wire(w.pin_mut());
        }
        Self(ffi::Wire_create(w.pin_mut()))
    }

    #[must_use]
    pub fn face(&self) -> Face {
        Face(ffi::Wire_face(&self.0))
    }

    #[must_use]
    pub fn build_curves_3d(mut self) -> Self {
        ffi::Wire_build_curves_3d(self.0.pin_mut());
        self
    }
}

impl Clone for Wire {
    fn clone(&self) -> Self {
        Self(ffi::Wire_clone(&self.0))
    }
}

impl fmt::Debug for Wire {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Wire").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Wire {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Wire {}

impl geom::Transformable for Wire {
    fn transform(&self, transformation: &geom::Transformation) -> Self {
        Self(ffi::Wire_transform(&self.0, &transformation.0))
    }
}

pub trait AddableToWire {
    fn add_to_wire(&self, maker: Pin<&mut ffi::WireBuilder>);
}

impl AddableToWire for Edge {
    fn add_to_wire(&self, mut maker: Pin<&mut ffi::WireBuilder>) {
        ffi::WireBuilder_add_edge(maker.as_mut(), &self.0);
    }
}

impl AddableToWire for Wire {
    fn add_to_wire(&self, mut maker: Pin<&mut ffi::WireBuilder>) {
        ffi::WireBuilder_add_wire(maker.as_mut(), &self.0);
    }
}

pub struct Loft(pub(crate) cxx::UniquePtr<ffi::Loft>);

impl Loft {
    #[must_use]
    pub fn new_solid() -> Self {
        Self(ffi::Loft_create_solid())
    }

    pub fn add_wires(&mut self, wire: &[&Wire]) -> &mut Self {
        for w in wire {
            ffi::Loft_add_wire(self.0.pin_mut(), &w.0);
        }
        self
    }

    pub fn ensure_wire_compatibility(&mut self, check: bool) -> &mut Self {
        ffi::Loft_ensure_wire_compatibility(self.0.pin_mut(), check);
        self
    }

    pub fn build(&mut self) -> Shape {
        Shape(ffi::Loft_build(self.0.pin_mut()))
    }
}

impl Clone for Loft {
    fn clone(&self) -> Self {
        Self(ffi::Loft_clone(&self.0))
    }
}

impl fmt::Debug for Loft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Loft").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Loft {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Loft {}

pub struct Compound(pub(crate) cxx::UniquePtr<ffi::Compound>);

impl Default for Compound {
    fn default() -> Self {
        Self::builder()
    }
}

impl Compound {
    #[must_use]
    pub fn builder() -> Self {
        let mut a = ffi::Compound_new();
        ffi::Compound_init(a.pin_mut());
        Self(a)
    }

    pub fn add(&mut self, shape: &Shape) -> &mut Self {
        ffi::Compound_add_shape(self.0.pin_mut(), &shape.0);
        self
    }

    pub fn build(&mut self) -> Shape {
        Shape(ffi::Compound_build(self.0.pin_mut()))
    }
}

impl fmt::Debug for Compound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: be more informative
        f.debug_struct("Loft").finish()
    }
}

// SAFETY: Safe because the underlying C++ type contains no thread-local state
// and all internal data is properly encapsulated.
unsafe impl Send for Compound {}

// SAFETY: Safe because this type provides no shared mutable access, and the underlying
// C++ type is designed for thread-safe read operations.
unsafe impl Sync for Compound {}
